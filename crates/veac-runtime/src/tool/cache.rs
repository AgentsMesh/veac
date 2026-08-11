use std::sync::{Condvar, Mutex};
use std::time::Instant;

#[derive(Debug)]
pub(crate) struct DeadlineCache<T> {
    state: Mutex<State<T>>,
    changed: Condvar,
}

#[derive(Debug)]
enum State<T> {
    Empty,
    Loading,
    Ready(T),
}

struct LoadingGuard<'a, T> {
    cache: &'a DeadlineCache<T>,
    active: bool,
}

#[derive(Debug)]
pub(crate) enum DeadlineCacheError<E> {
    Deadline,
    Poisoned,
    Initialization(E),
}

impl<T> Default for DeadlineCache<T> {
    fn default() -> Self {
        Self {
            state: Mutex::new(State::Empty),
            changed: Condvar::new(),
        }
    }
}

impl<T: Clone> DeadlineCache<T> {
    pub(crate) fn get_or_try_init<E>(
        &self,
        deadline: Instant,
        initialize: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, DeadlineCacheError<E>> {
        let mut initialize = Some(initialize);
        let mut state = self
            .state
            .lock()
            .map_err(|_| DeadlineCacheError::Poisoned)?;
        loop {
            let remaining = deadline
                .checked_duration_since(Instant::now())
                .filter(|value| !value.is_zero())
                .ok_or(DeadlineCacheError::Deadline)?;
            match &*state {
                State::Ready(value) => return Ok(value.clone()),
                State::Empty => {
                    *state = State::Loading;
                    drop(state);
                    let mut loading = LoadingGuard::new(self);
                    let result = initialize.take().expect("initializer is retained")();
                    let finished = self.finish(result);
                    loading.disarm();
                    return finished;
                }
                State::Loading => {
                    let waited = self
                        .changed
                        .wait_timeout(state, remaining)
                        .map_err(|_| DeadlineCacheError::Poisoned)?;
                    state = waited.0;
                }
            }
        }
    }

    fn finish<E>(&self, result: Result<T, E>) -> Result<T, DeadlineCacheError<E>> {
        let mut state = self
            .state
            .lock()
            .map_err(|_| DeadlineCacheError::Poisoned)?;
        match result {
            Ok(value) => {
                *state = State::Ready(value.clone());
                self.changed.notify_all();
                Ok(value)
            }
            Err(error) => {
                *state = State::Empty;
                self.changed.notify_all();
                Err(DeadlineCacheError::Initialization(error))
            }
        }
    }
}

impl<'a, T> LoadingGuard<'a, T> {
    fn new(cache: &'a DeadlineCache<T>) -> Self {
        Self {
            cache,
            active: true,
        }
    }

    fn disarm(&mut self) {
        self.active = false;
    }
}

impl<T> Drop for LoadingGuard<'_, T> {
    fn drop(&mut self) {
        if self.active {
            if let Ok(mut state) = self.cache.state.lock() {
                *state = State::Empty;
                self.cache.changed.notify_all();
            }
        }
    }
}

#[cfg(test)]
mod tests;

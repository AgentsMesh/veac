use std::process::{Child, Command};
use std::time::{Duration, Instant};

const RETRY_INTERVAL: Duration = Duration::from_millis(2);
const RETRY_WINDOW: Duration = Duration::from_millis(250);

#[derive(Debug)]
pub(crate) enum PinnedSpawnError {
    Deadline,
    Io(std::io::Error),
}

pub(crate) fn spawn_pinned_until(
    command: &mut Command,
    deadline: Instant,
) -> Result<Child, PinnedSpawnError> {
    retry_until(
        deadline,
        Instant::now,
        || command.spawn(),
        std::thread::sleep,
    )
}

fn retry_until<T>(
    deadline: Instant,
    mut now: impl FnMut() -> Instant,
    mut launch: impl FnMut() -> std::io::Result<T>,
    mut sleep: impl FnMut(Duration),
) -> Result<T, PinnedSpawnError> {
    let started = now();
    if started >= deadline {
        return Err(PinnedSpawnError::Deadline);
    }
    let retry_limit = started + RETRY_WINDOW;
    loop {
        match launch() {
            Ok(child) => return Ok(child),
            Err(error) if text_busy(&error) => {
                let current = now();
                if current >= deadline {
                    return Err(PinnedSpawnError::Deadline);
                }
                if current >= retry_limit {
                    return Err(PinnedSpawnError::Io(error));
                }
                let remaining = deadline.min(retry_limit) - current;
                sleep(RETRY_INTERVAL.min(remaining));
                let current = now();
                if current >= deadline {
                    return Err(PinnedSpawnError::Deadline);
                }
                if current >= retry_limit {
                    return Err(PinnedSpawnError::Io(error));
                }
            }
            Err(error) => return Err(PinnedSpawnError::Io(error)),
        }
    }
}

fn text_busy(error: &std::io::Error) -> bool {
    error.raw_os_error() == Some(rustix::io::Errno::TXTBSY.raw_os_error())
}

#[cfg(test)]
mod tests;

use super::{ExecutionBudget, RESOURCES, RESOURCE_COUNT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Checkpoint {
    used: [usize; RESOURCE_COUNT],
}

impl ExecutionBudget {
    pub(crate) fn transaction<T, E>(
        &self,
        operation: impl FnOnce() -> Result<T, E>,
    ) -> Result<T, E> {
        let checkpoint = self.checkpoint();
        match operation() {
            Ok(value) => Ok(value),
            Err(error) => {
                self.restore(checkpoint);
                Err(error)
            }
        }
    }

    fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            used: RESOURCES.map(|resource| self.counters[resource.index()].used.get()),
        }
    }

    fn restore(&self, checkpoint: Checkpoint) {
        for resource in RESOURCES {
            self.counters[resource.index()]
                .used
                .set(checkpoint.used[resource.index()]);
        }
    }
}

#[cfg(test)]
#[path = "transaction/test_api.rs"]
mod test_api;

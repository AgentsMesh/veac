use std::time::{Duration, Instant};

use crate::RuntimeError;

pub const MAX_TASK_WALL_TIME: Duration =
    Duration::from_secs(veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskExecutionLimits {
    max_wall_time: Duration,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BundleSetupLimits {
    max_wall_time: Duration,
}

impl TaskExecutionLimits {
    pub fn new(max_wall_time: Duration) -> Result<Self, RuntimeError> {
        validate(max_wall_time, "task")?;
        Ok(Self { max_wall_time })
    }

    pub fn max_wall_time(self) -> Duration {
        self.max_wall_time
    }

    pub(super) fn deadline(self) -> Result<Instant, RuntimeError> {
        deadline_from(self.max_wall_time)
    }
}

impl BundleSetupLimits {
    pub fn new(max_wall_time: Duration) -> Result<Self, RuntimeError> {
        validate(max_wall_time, "bundle setup")?;
        Ok(Self { max_wall_time })
    }

    pub fn max_wall_time(self) -> Duration {
        self.max_wall_time
    }

    pub(super) fn deadline(self) -> Result<Instant, RuntimeError> {
        deadline_from(self.max_wall_time)
    }
}

impl Default for TaskExecutionLimits {
    fn default() -> Self {
        Self {
            max_wall_time: MAX_TASK_WALL_TIME,
        }
    }
}

impl Default for BundleSetupLimits {
    fn default() -> Self {
        Self {
            max_wall_time: MAX_TASK_WALL_TIME,
        }
    }
}

pub(super) fn ensure(deadline: Instant) -> Result<(), RuntimeError> {
    if Instant::now() < deadline {
        Ok(())
    } else {
        Err(RuntimeError::resource_limit(
            "backend task exceeded its wall-clock limit",
        ))
    }
}

pub(super) fn ensure_setup(deadline: Instant) -> Result<(), RuntimeError> {
    if Instant::now() < deadline {
        Ok(())
    } else {
        Err(RuntimeError::resource_limit(
            "bundle setup exceeded its wall-clock limit",
        ))
    }
}

fn validate(max_wall_time: Duration, scope: &str) -> Result<(), RuntimeError> {
    if max_wall_time.is_zero() || max_wall_time > MAX_TASK_WALL_TIME {
        return Err(RuntimeError::new(format!(
            "{scope} wall time must be positive and at most the hard execution limit"
        )));
    }
    Ok(())
}

fn deadline_from(max_wall_time: Duration) -> Result<Instant, RuntimeError> {
    Instant::now()
        .checked_add(max_wall_time)
        .ok_or_else(|| RuntimeError::resource_limit("execution wall-clock deadline overflowed"))
}

#[cfg(test)]
#[path = "deadline/tests.rs"]
mod tests;

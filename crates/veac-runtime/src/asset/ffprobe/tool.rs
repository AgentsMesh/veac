use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use crate::tool::PinnedExecutable;
use crate::RuntimeError;

#[derive(Debug, Clone)]
pub struct SystemFfprobe {
    binary: PathBuf,
    pinned: Arc<OnceLock<PinnedExecutable>>,
}

impl SystemFfprobe {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            pinned: Arc::new(OnceLock::new()),
        }
    }

    pub(super) fn binary(&self) -> &Path {
        &self.binary
    }

    pub(super) fn pinned_until(
        &self,
        deadline: Instant,
    ) -> Result<&PinnedExecutable, RuntimeError> {
        PinnedExecutable::cached_until(&self.pinned, &self.binary, deadline).map_err(|error| {
            error.context(&format!("failed to run ffprobe {}", self.binary.display()))
        })
    }
}

impl Default for SystemFfprobe {
    fn default() -> Self {
        Self::new("ffprobe")
    }
}

#[cfg(test)]
#[path = "tool/tests.rs"]
mod tests;

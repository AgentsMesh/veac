use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Instant;

use super::{version, ProbeError};
use crate::tool::{DeadlineCache, DeadlineCacheError, PinnedExecutable};
use crate::RuntimeError;

#[derive(Debug, Clone)]
pub struct SystemFfprobe {
    binary: PathBuf,
    pinned: Arc<OnceLock<PinnedExecutable>>,
    version: Arc<DeadlineCache<String>>,
}

impl SystemFfprobe {
    pub fn new(binary: impl Into<PathBuf>) -> Self {
        Self {
            binary: binary.into(),
            pinned: Arc::new(OnceLock::new()),
            version: Arc::new(DeadlineCache::default()),
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

    pub(super) fn version_until(
        &self,
        pinned: &PinnedExecutable,
        deadline: Instant,
    ) -> Result<String, ProbeError> {
        self.version
            .get_or_try_init(deadline, || version::read(&self.binary, pinned, deadline))
            .map_err(|error| self.version_cache_error(error))
    }

    fn version_cache_error(&self, error: DeadlineCacheError<ProbeError>) -> ProbeError {
        match error {
            DeadlineCacheError::Deadline => ProbeError::ResourceLimit {
                operation: "version cache",
            },
            DeadlineCacheError::Poisoned => version::tool_error(
                &self.binary,
                RuntimeError::new("ffprobe version cache is unavailable"),
            ),
            DeadlineCacheError::Initialization(error) => error,
        }
    }
}

impl Default for SystemFfprobe {
    fn default() -> Self {
        Self::new("ffprobe")
    }
}

#[cfg(test)]
#[path = "tool/cache_tests.rs"]
mod cache_tests;
#[cfg(test)]
#[path = "tool/tests.rs"]
mod tests;

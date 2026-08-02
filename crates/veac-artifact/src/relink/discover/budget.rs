use std::path::PathBuf;
use std::time::Instant;

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, RelinkCandidate};

use super::RelinkDiscoveryLimits;

pub(super) struct DiscoveryBudget {
    limits: RelinkDiscoveryLimits,
    entries: usize,
    hashed_bytes: u64,
    deadline: Instant,
}

impl DiscoveryBudget {
    pub(super) fn new(limits: RelinkDiscoveryLimits, started: Instant) -> ArtifactResult<Self> {
        let deadline = started
            .checked_add(limits.max_wall_time)
            .ok_or_else(|| limit_error("relink discovery deadline overflowed"))?;
        Ok(Self {
            limits,
            entries: 0,
            hashed_bytes: 0,
            deadline,
        })
    }

    pub(super) fn max_depth(&self) -> usize {
        self.limits.max_depth
    }

    pub(super) fn observe_entry(&mut self) -> ArtifactResult<()> {
        self.check_deadline()?;
        self.entries = self
            .entries
            .checked_add(1)
            .ok_or_else(|| limit_error("relink discovery entry count overflowed"))?;
        if self.entries > self.limits.max_entries {
            return limit("relink discovery exceeds the entry budget");
        }
        Ok(())
    }

    pub(super) fn file_limit(&self) -> ArtifactResult<u64> {
        self.check_deadline()?;
        let remaining = self
            .limits
            .max_total_bytes
            .checked_sub(self.hashed_bytes)
            .ok_or_else(|| limit_error("relink discovery byte budget underflowed"))?;
        Ok(self.limits.max_file_bytes.min(remaining).max(1))
    }

    pub(super) fn candidate(
        &mut self,
        path: PathBuf,
        verified: crate::VerifiedSourceCopy,
    ) -> ArtifactResult<RelinkCandidate> {
        self.check_deadline()?;
        self.hashed_bytes = self
            .hashed_bytes
            .checked_add(verified.size_bytes)
            .ok_or_else(|| limit_error("relink discovery hashed byte count overflowed"))?;
        if self.hashed_bytes > self.limits.max_total_bytes {
            return limit("relink discovery exceeds the aggregate byte budget");
        }
        Ok(RelinkCandidate {
            path,
            identity: verified.identity,
        })
    }

    pub(super) fn check_deadline(&self) -> ArtifactResult<()> {
        if Instant::now() < self.deadline {
            Ok(())
        } else {
            limit("relink discovery exceeded its wall-clock budget")
        }
    }

    pub(super) fn before_deadline(&self) -> bool {
        Instant::now() < self.deadline
    }
}

fn limit<T>(message: &str) -> ArtifactResult<T> {
    Err(limit_error(message))
}

fn limit_error(message: &str) -> ArtifactError {
    ArtifactError::new(ArtifactErrorKind::ResourceLimit, message)
}

#[cfg(test)]
#[path = "budget/tests.rs"]
mod tests;

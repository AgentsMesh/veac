use std::time::Duration;

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

pub const MAX_RELINK_DISCOVERY_DEPTH: usize = 64;
pub const MAX_RELINK_DISCOVERY_ENTRIES: usize = 65_536;
pub const MAX_RELINK_DISCOVERY_FILE_BYTES: u64 = crate::MAX_VERIFIED_SOURCE_BYTES;
pub const MAX_RELINK_DISCOVERY_TOTAL_BYTES: u64 = 4 * crate::MAX_VERIFIED_SOURCE_BYTES;
pub const MAX_RELINK_DISCOVERY_WALL_TIME: Duration = Duration::from_secs(6 * 60 * 60);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RelinkDiscoveryLimits {
    pub max_depth: usize,
    pub max_entries: usize,
    pub max_file_bytes: u64,
    pub max_total_bytes: u64,
    pub max_wall_time: Duration,
}

impl Default for RelinkDiscoveryLimits {
    fn default() -> Self {
        Self {
            max_depth: MAX_RELINK_DISCOVERY_DEPTH,
            max_entries: MAX_RELINK_DISCOVERY_ENTRIES,
            max_file_bytes: MAX_RELINK_DISCOVERY_FILE_BYTES,
            max_total_bytes: MAX_RELINK_DISCOVERY_TOTAL_BYTES,
            max_wall_time: MAX_RELINK_DISCOVERY_WALL_TIME,
        }
    }
}

impl RelinkDiscoveryLimits {
    pub(super) fn validate(self) -> ArtifactResult<()> {
        let hard = Self::default();
        if self.max_depth > hard.max_depth
            || self.max_entries == 0
            || self.max_entries > hard.max_entries
            || self.max_file_bytes == 0
            || self.max_file_bytes > hard.max_file_bytes
            || self.max_total_bytes == 0
            || self.max_total_bytes > hard.max_total_bytes
            || self.max_wall_time.is_zero()
            || self.max_wall_time > hard.max_wall_time
        {
            return Err(ArtifactError::new(
                ArtifactErrorKind::InvalidContract,
                "relink discovery limits are outside the supported policy",
            ));
        }
        Ok(())
    }
}

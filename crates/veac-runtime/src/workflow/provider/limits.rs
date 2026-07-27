use std::time::Duration;

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

pub const MAX_PROVIDER_PROTOCOL_BYTES: u64 = 16 * 1024 * 1024;
pub const MAX_PROVIDER_STDERR_BYTES: u64 = 1024 * 1024;
pub const MAX_PROVIDER_WALL_TIME: Duration = Duration::from_secs(6 * 60 * 60);
pub const MAX_PROVIDER_ARTIFACTS: usize = 4_096;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProviderResourceLimits {
    pub max_request_bytes: u64,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
    pub max_wall_time: Duration,
    pub max_artifacts: usize,
    pub max_payload_bytes: u64,
    pub max_total_payload_bytes: u64,
}

impl Default for ProviderResourceLimits {
    fn default() -> Self {
        Self {
            max_request_bytes: MAX_PROVIDER_PROTOCOL_BYTES,
            max_stdout_bytes: MAX_PROVIDER_PROTOCOL_BYTES,
            max_stderr_bytes: MAX_PROVIDER_STDERR_BYTES,
            max_wall_time: MAX_PROVIDER_WALL_TIME,
            max_artifacts: MAX_PROVIDER_ARTIFACTS,
            max_payload_bytes: veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES,
            max_total_payload_bytes: veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES,
        }
    }
}

impl ProviderResourceLimits {
    pub(super) fn validate(self) -> WorkflowResult<()> {
        let hard = Self::default();
        if self.max_request_bytes == 0
            || self.max_request_bytes > hard.max_request_bytes
            || self.max_stdout_bytes == 0
            || self.max_stdout_bytes > hard.max_stdout_bytes
            || self.max_stderr_bytes == 0
            || self.max_stderr_bytes > hard.max_stderr_bytes
            || self.max_wall_time.is_zero()
            || self.max_wall_time > hard.max_wall_time
            || self.max_artifacts == 0
            || self.max_artifacts > hard.max_artifacts
            || self.max_payload_bytes == 0
            || self.max_payload_bytes > hard.max_payload_bytes
            || self.max_total_payload_bytes == 0
            || self.max_total_payload_bytes > hard.max_total_payload_bytes
        {
            return Err(WorkflowError::new(
                WorkflowErrorKind::InvalidContract,
                "provider resource limits are outside the supported policy",
            ));
        }
        Ok(())
    }
}

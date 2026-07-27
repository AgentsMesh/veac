use std::path::Path;
use std::time::{Duration, Instant};

use veac_artifact::{ArtifactRecord, FullRenderSegmentContract};
use veac_ir::{HashAlgorithm, MediaIdentity, MediaProbeSnapshot, StreamChoice, StreamIntent};

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};
use crate::asset::{ProbeError, SystemFfprobe};

mod audio;
mod container;
mod contract;
mod payload;
mod video;

#[cfg(test)]
#[path = "render_segment/tests.rs"]
mod tests;

#[derive(Debug, Clone)]
pub struct FullRenderSegmentValidator {
    ffprobe: SystemFfprobe,
    max_wall_seconds: u64,
    max_payload_bytes: u64,
}

impl FullRenderSegmentValidator {
    pub fn new(ffprobe: SystemFfprobe) -> Self {
        Self {
            ffprobe,
            max_wall_seconds: veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS,
            max_payload_bytes: veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES,
        }
    }

    pub fn with_limits(
        ffprobe: SystemFfprobe,
        max_wall_seconds: u64,
        max_payload_bytes: u64,
    ) -> WorkflowResult<Self> {
        let value = Self {
            ffprobe,
            max_wall_seconds,
            max_payload_bytes,
        };
        value.validate_limits()?;
        Ok(value)
    }

    pub fn validate(
        &self,
        path: &Path,
        segment: &FullRenderSegmentContract,
        expected: &ArtifactRecord,
    ) -> WorkflowResult<()> {
        self.validate_limits()?;
        let deadline = Instant::now()
            .checked_add(Duration::from_secs(self.max_wall_seconds))
            .expect("validated render-segment wall budget fits an Instant");
        self.validate_until(path, segment, expected, deadline)
    }

    pub fn validate_until(
        &self,
        path: &Path,
        segment: &FullRenderSegmentContract,
        expected: &ArtifactRecord,
        caller_deadline: Instant,
    ) -> WorkflowResult<()> {
        self.validate_limits()?;
        contract::validate_record(segment, expected, self.max_payload_bytes)?;
        let policy_deadline = Instant::now()
            .checked_add(Duration::from_secs(self.max_wall_seconds))
            .expect("validated render-segment wall budget fits an Instant");
        let deadline = caller_deadline.min(policy_deadline);
        check_deadline(deadline)?;
        let identity = expected_identity(expected);
        payload::verify(path, &identity, expected, self.max_payload_bytes, deadline)?;
        check_deadline(deadline)?;
        let intent = stream_intent(segment.has_audio());
        let probe = self
            .ffprobe
            .probe_with_intent_until(path, intent, deadline)
            .map_err(probe_error)?;
        let verified =
            payload::verify(path, &identity, expected, self.max_payload_bytes, deadline)?;
        container::validate_prefix(&verified.prefix, segment.media_profile().video().container)?;
        check_deadline(deadline)?;
        validate_full_render_segment_snapshot(segment, expected, &probe)
    }

    fn validate_limits(&self) -> WorkflowResult<()> {
        if self.max_wall_seconds == 0
            || self.max_wall_seconds > veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS
            || self.max_payload_bytes == 0
            || self.max_payload_bytes > veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES
        {
            return Err(contract_error(
                "render-segment limits exceed the hard policy",
            ));
        }
        Ok(())
    }
}

pub fn validate_full_render_segment_snapshot(
    segment: &FullRenderSegmentContract,
    expected: &ArtifactRecord,
    probe: &MediaProbeSnapshot,
) -> WorkflowResult<()> {
    contract::validate_record(segment, expected, veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES)?;
    if probe.observed_identity != expected_identity(expected) {
        return Err(identity_error(
            "render-segment probe identity differs from its record",
        ));
    }
    contract::validate_snapshot(segment, probe)
}

pub fn validate_full_render_segment_prefix(
    segment: &FullRenderSegmentContract,
    prefix: &[u8],
) -> WorkflowResult<()> {
    container::validate_prefix(prefix, segment.media_profile().video().container)
}

fn stream_intent(has_audio: bool) -> StreamIntent {
    StreamIntent {
        video: StreamChoice::Auto,
        audio: if has_audio {
            StreamChoice::Auto
        } else {
            StreamChoice::Disabled
        },
    }
}

fn expected_identity(record: &ArtifactRecord) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: record.content.value.clone(),
    }
}

fn probe_error(error: ProbeError) -> WorkflowError {
    let kind = match error {
        ProbeError::ResourceLimit { .. } => WorkflowErrorKind::ResourceLimit,
        ProbeError::IdentityChanged { .. } => WorkflowErrorKind::SourceIdentityMismatch,
        _ => WorkflowErrorKind::ToolFailure,
    };
    WorkflowError::with_source(kind, "render-segment cannot be probed", error)
}

fn check_deadline(deadline: Instant) -> WorkflowResult<()> {
    if Instant::now() >= deadline {
        return Err(resource_error(
            "render-segment validation exceeded its wall budget",
        ));
    }
    Ok(())
}

pub(super) fn contract_error(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::InvalidContract, message)
}

pub(super) fn media_error(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::ToolFailure, message)
}

fn identity_error(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::SourceIdentityMismatch, message)
}

fn resource_error(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::ResourceLimit, message)
}

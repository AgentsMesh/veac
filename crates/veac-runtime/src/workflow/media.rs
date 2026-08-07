use std::path::Path;
use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use veac_artifact::{
    AnalysisIngestionRequest, ArtifactError, ArtifactErrorKind, ArtifactRecord, ArtifactStore,
    MediaArtifactLimits, MediaArtifactSpec,
};
use veac_ir::MediaProbeSnapshot;

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};
use crate::asset::SystemFfprobe;
use crate::executor::SystemFfmpeg;

mod analysis;
mod cache;
mod command;
mod derive;
#[cfg(test)]
#[path = "media/error_mapping_tests.rs"]
mod error_mapping_tests;
#[cfg(test)]
#[path = "media/failure_tests.rs"]
mod failure_tests;
mod output;
mod postflight;
#[cfg(test)]
#[path = "media/postflight_contract_tests.rs"]
mod postflight_contract_tests;
mod preflight;
mod process;
#[cfg(test)]
#[path = "media/resource_tests.rs"]
mod resource_tests;
#[cfg(test)]
#[path = "media/security_tests.rs"]
mod security_tests;
mod source;
#[cfg(test)]
#[path = "media/test_support.rs"]
mod test_support;
#[cfg(test)]
#[path = "media/tests.rs"]
mod tests;
mod tool;

pub(crate) use postflight::{audio_duration_matches, video_duration_matches};
pub use tool::media_artifact_producer;

pub fn validate_media_artifact_snapshot(
    probe: &MediaProbeSnapshot,
    spec: &MediaArtifactSpec,
) -> WorkflowResult<()> {
    postflight::validate_snapshot(probe, spec)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeneratedArtifact {
    pub record: ArtifactRecord,
    pub cache_hit: bool,
}

#[derive(Debug, Clone)]
pub struct MediaWorkflow {
    ffmpeg: SystemFfmpeg,
    ffprobe: SystemFfprobe,
    limits: MediaArtifactLimits,
}

impl MediaWorkflow {
    pub fn new(ffmpeg: impl Into<std::path::PathBuf>) -> Self {
        Self::with_tools(ffmpeg, "ffprobe")
    }

    pub fn with_tools(
        ffmpeg: impl Into<std::path::PathBuf>,
        ffprobe: impl Into<std::path::PathBuf>,
    ) -> Self {
        Self {
            ffmpeg: SystemFfmpeg::new(ffmpeg),
            ffprobe: SystemFfprobe::new(ffprobe),
            limits: MediaArtifactLimits::default(),
        }
    }

    pub fn with_limits(mut self, limits: MediaArtifactLimits) -> Self {
        self.limits = limits;
        self
    }

    pub fn ingest_analysis(
        &self,
        store: &ArtifactStore,
        input: &Path,
        request: &AnalysisIngestionRequest,
    ) -> WorkflowResult<GeneratedArtifact> {
        let deadline =
            Instant::now() + Duration::from_secs(self.limits.max_derivation_wall_seconds);
        analysis::store(store, input, request, self.limits, deadline)
    }
}

fn contract<T>(value: veac_artifact::ArtifactResult<T>) -> WorkflowResult<T> {
    value.map_err(|error| {
        let kind = match error.kind {
            ArtifactErrorKind::ResourceLimit => WorkflowErrorKind::ResourceLimit,
            ArtifactErrorKind::InvalidContract | ArtifactErrorKind::UnsupportedIdentity => {
                WorkflowErrorKind::InvalidContract
            }
            _ => WorkflowErrorKind::Artifact,
        };
        WorkflowError::with_source(kind, "media artifact contract is invalid", error)
    })
}

pub(super) fn launch_error(error: crate::RuntimeError) -> WorkflowError {
    let kind = if error.kind == crate::RuntimeErrorKind::ResourceLimit {
        WorkflowErrorKind::ResourceLimit
    } else {
        WorkflowErrorKind::ToolFailure
    };
    WorkflowError::with_source(kind, "failed to prepare FFmpeg artifact derivation", error)
}

fn artifact_error(error: ArtifactError) -> WorkflowError {
    let kind = if error.kind == ArtifactErrorKind::ResourceLimit {
        WorkflowErrorKind::ResourceLimit
    } else {
        WorkflowErrorKind::Artifact
    };
    WorkflowError::with_source(kind, "artifact store operation failed", error)
}

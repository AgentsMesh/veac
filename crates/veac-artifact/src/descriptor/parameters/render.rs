use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{DeliverableId, SequenceId, TimeRange};

use crate::{ArtifactResult, ContentDigest, MAX_ARTIFACT_JSON_STRING_BYTES};

use super::super::{invalid, resource_limit};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RenderSegmentFidelity {
    ExactDeliveryMaster,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderSegmentParameters {
    pub sequence_id: SequenceId,
    pub range: TimeRange,
    pub deliverable_id: DeliverableId,
    pub fidelity: RenderSegmentFidelity,
}

impl RenderSegmentParameters {
    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        if !self.range.start.is_valid()
            || !self.range.duration.is_valid()
            || self.range.start.value < 0
            || self.range.duration.value <= 0
        {
            return invalid("render segment parameters contain an invalid range");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderOutputParameters {
    pub index: u64,
    pub path: String,
}

impl RenderOutputParameters {
    pub fn new(index: usize, path: impl Into<String>) -> Self {
        Self {
            index: u64::try_from(index).unwrap_or(u64::MAX),
            path: path.into(),
        }
    }

    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        if self.path.is_empty() || self.path.bytes().any(|byte| byte == 0) {
            return invalid("render output path must be non-empty and contain no NUL byte");
        }
        if self.path.len() > MAX_ARTIFACT_JSON_STRING_BYTES {
            return resource_limit("render output path exceeds its byte budget");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RenderTaskPhase {
    Single,
    FirstPass,
    SecondPass,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RenderProduct {
    VideoMaster,
    RenderPassLog,
    ImageSequence,
    CaptionSidecar,
    AudioStem,
    AudioFile,
    AnimatedImage,
    StillImage,
    HlsVod,
    VideoWaveform,
    Vectorscope,
    Histogram,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderTaskParameters {
    pub contract_version: u32,
    pub deliverable_id: DeliverableId,
    pub phase: RenderTaskPhase,
    pub product: RenderProduct,
    pub task_digest: ContentDigest,
}

impl RenderTaskParameters {
    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        if self.contract_version == 0 {
            return invalid("render task contract version must be non-zero");
        }
        self.task_digest.validate()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "scope",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum RenderCheckpointParameters {
    Task(RenderTaskParameters),
    Output(RenderOutputParameters),
}

impl RenderCheckpointParameters {
    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        match self {
            Self::Task(value) => value.validate(),
            Self::Output(value) => value.validate(),
        }
    }
}

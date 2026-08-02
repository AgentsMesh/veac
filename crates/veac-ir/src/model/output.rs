use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AdaptivePackage, AnimatedImage, AudioFile, AudioStemOutput, CaptionSidecarOutput,
    DeliverableId, ImageSequenceOutput, Rational, RenderConfigId, ScopeOutput, SequenceId,
    StillImage, VideoDeliverable,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderConfig {
    pub id: RenderConfigId,
    pub sequence_id: SequenceId,
    pub raster: Option<RasterSettings>,
    pub deliverables: Vec<Deliverable>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RasterSettings {
    pub width: u32,
    pub height: u32,
    pub frame_rate: Rational,
    pub captions: CaptionOutput,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Deliverable {
    pub id: DeliverableId,
    pub target: DeliverableTarget,
    pub kind: DeliverableKind,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum DeliverableTarget {
    File { name: String },
    ImageSequence { pattern: String },
    Package { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "settings",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum DeliverableKind {
    Video(VideoDeliverable),
    ImageSequence(ImageSequenceOutput),
    CaptionSidecar(CaptionSidecarOutput),
    AudioStem(AudioStemOutput),
    Scope(ScopeOutput),
    AudioFile(AudioFile),
    AnimatedImage(AnimatedImage),
    StillImage(StillImage),
    AdaptivePackage(AdaptivePackage),
}

impl RenderConfig {
    pub fn video_deliverable(&self, id: &DeliverableId) -> Option<&VideoDeliverable> {
        self.deliverables
            .iter()
            .find(|value| value.id == *id)
            .and_then(|value| match &value.kind {
                DeliverableKind::Video(settings) => Some(settings),
                _ => None,
            })
    }

    pub fn video_deliverable_mut(&mut self, id: &DeliverableId) -> Option<&mut VideoDeliverable> {
        self.deliverables
            .iter_mut()
            .find(|value| value.id == *id)
            .and_then(|value| match &mut value.kind {
                DeliverableKind::Video(settings) => Some(settings),
                _ => None,
            })
    }
}

impl DeliverableKind {
    pub fn requires_raster(&self) -> bool {
        matches!(
            self,
            Self::Video(_)
                | Self::ImageSequence(_)
                | Self::Scope(_)
                | Self::AnimatedImage(_)
                | Self::StillImage(_)
                | Self::AdaptivePackage(_)
        )
    }
}

impl DeliverableTarget {
    pub fn file_name(&self) -> Option<&str> {
        match self {
            Self::File { name } => Some(name),
            Self::ImageSequence { .. } | Self::Package { .. } => None,
        }
    }

    pub fn image_sequence_pattern(&self) -> Option<&str> {
        match self {
            Self::ImageSequence { pattern } => Some(pattern),
            Self::File { .. } | Self::Package { .. } => None,
        }
    }

    pub fn package_name(&self) -> Option<&str> {
        match self {
            Self::Package { name } => Some(name),
            Self::File { .. } | Self::ImageSequence { .. } => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AudioOutput {
    pub codec: AudioCodec,
    pub sample_rate: u32,
    pub channels: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CaptionOutput {
    BurnIn,
    Discard,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OutputFormat {
    Mp4,
    Mov,
    Mkv,
    Webm,
    Mxf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AudioCodec {
    Aac,
    Opus,
    Flac,
    PcmS16Le,
    PcmS24Le,
    PcmS32Le,
}

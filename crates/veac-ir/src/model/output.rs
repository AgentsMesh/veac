use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AudioStemOutput, CaptionSidecarOutput, DeliverableId, ImageSequenceOutput, Rational,
    RenderConfigId, ScopeOutput, SequenceId, VideoDeliverable,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RenderConfig {
    pub id: RenderConfigId,
    pub sequence_id: SequenceId,
    pub width: u32,
    pub height: u32,
    pub frame_rate: Rational,
    pub deliverables: Vec<Deliverable>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Deliverable {
    pub id: DeliverableId,
    pub file_name: String,
    pub kind: DeliverableKind,
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
}

impl RenderConfig {
    pub fn video_deliverable(&self) -> Option<&VideoDeliverable> {
        self.deliverables
            .iter()
            .find_map(|value| match &value.kind {
                DeliverableKind::Video(settings) => Some(settings),
                _ => None,
            })
    }

    pub fn video_deliverable_mut(&mut self) -> Option<&mut VideoDeliverable> {
        self.deliverables
            .iter_mut()
            .find_map(|value| match &mut value.kind {
                DeliverableKind::Video(settings) => Some(settings),
                _ => None,
            })
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

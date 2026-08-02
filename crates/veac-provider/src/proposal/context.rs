use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    ColorSpace, EffectId, ItemId, OperationId, RationalTime, SequenceId, TextStyle, TrackId,
    VisualProperties,
};

use crate::Capability;

#[path = "context/media.rs"]
mod media;
pub use media::*;
#[path = "context/annotation.rs"]
mod annotation;
pub use annotation::*;
#[path = "context/visual.rs"]
mod visual;
pub use visual::*;
#[path = "context/multicam.rs"]
mod multicam;
pub use multicam::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum ApplicationContext {
    AsrCaptions(Box<AsrCaptionApplication>),
    Translation(TranslationApplication),
    AnalysisAnnotations(AnnotationApplication),
    TextToSpeech(Box<GeneratedAudioApplication>),
    Dubbing(Box<DubbingApplication>),
    MotionTracking(TransformApplication),
    Stabilization(TransformApplication),
    Segmentation(Box<MatteApplication>),
    Matte(Box<MatteApplication>),
    Denoise(Box<MediaReplacementApplication>),
    VocalSeparation(Box<SeparationApplication>),
    AutoReframe(AutoReframeApplication),
    Removal(Box<MediaReplacementApplication>),
    Retouch(Box<RetouchApplication>),
    ColorMatch(ColorMatchApplication),
    MulticamSync(MulticamSyncApplication),
}

/// Retained as a Rust source alias; serialized contexts use the tagged enum above.
pub type CaptionProposalContext = ApplicationContext;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApplicationHeader {
    #[schemars(range(max = 9007199254740991u64))]
    pub project_revision: u64,
    pub operation_id: OperationId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AsrCaptionApplication {
    pub header: ApplicationHeader,
    pub sequence_id: SequenceId,
    pub track_id: TrackId,
    pub style: TextStyle,
    pub visual: VisualProperties,
    pub item_id_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TranslationBinding {
    pub unit_id: String,
    pub clip_id: ItemId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TranslationApplication {
    pub header: ApplicationHeader,
    pub bindings: Vec<TranslationBinding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ClipTimeBinding {
    pub provider_origin: RationalTime,
    pub clip_local_origin: RationalTime,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransformApplication {
    pub header: ApplicationHeader,
    pub clip_id: ItemId,
    pub time: ClipTimeBinding,
    pub keyframe_id_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AutoReframeApplication {
    pub header: ApplicationHeader,
    pub clip_id: ItemId,
    pub time: ClipTimeBinding,
    pub keyframe_id_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ColorMatchApplication {
    pub header: ApplicationHeader,
    pub clip_id: ItemId,
    pub color_space: ColorSpace,
    pub effect_id: EffectId,
}

impl ApplicationContext {
    pub fn capability(&self) -> Capability {
        match self {
            Self::AsrCaptions(_) => Capability::Asr,
            Self::Translation(_) => Capability::Translation,
            Self::AnalysisAnnotations(value) => value.capability,
            Self::TextToSpeech(_) => Capability::TextToSpeech,
            Self::Dubbing(_) => Capability::Dubbing,
            Self::MotionTracking(_) => Capability::MotionTracking,
            Self::Stabilization(_) => Capability::Stabilization,
            Self::Segmentation(_) => Capability::Segmentation,
            Self::Matte(_) => Capability::Matte,
            Self::Denoise(_) => Capability::Denoise,
            Self::VocalSeparation(_) => Capability::VocalSeparation,
            Self::AutoReframe(_) => Capability::AutoReframe,
            Self::Removal(_) => Capability::Removal,
            Self::Retouch(_) => Capability::Retouch,
            Self::ColorMatch(_) => Capability::ColorMatch,
            Self::MulticamSync(_) => Capability::MulticamSync,
        }
    }

    pub(crate) fn header(&self) -> &ApplicationHeader {
        match self {
            Self::AsrCaptions(value) => &value.header,
            Self::Translation(value) => &value.header,
            Self::AnalysisAnnotations(value) => &value.header,
            Self::TextToSpeech(value) => &value.header,
            Self::Dubbing(value) => &value.header,
            Self::MotionTracking(value) | Self::Stabilization(value) => &value.header,
            Self::Segmentation(value) | Self::Matte(value) => &value.header,
            Self::Denoise(value) | Self::Removal(value) => &value.header,
            Self::VocalSeparation(value) => &value.header,
            Self::AutoReframe(value) => &value.header,
            Self::Retouch(value) => &value.header,
            Self::ColorMatch(value) => &value.header,
            Self::MulticamSync(value) => &value.header,
        }
    }
}

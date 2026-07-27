use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    Animatable, AudioCrossfade, AudioProcessor, CardStyle, Compositing, Frame, ItemId, Mask,
    PitchPolicy, Placement, RelationId, SidechainSource, TimeRange, TrackMatteMode, Transform2D,
};

use super::ResolvedColorPipeline;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectiveVisualProperties {
    /// Static layout base in the sequence canvas. Emitters resolve this before transforms.
    pub placement: Placement,
    pub frame: Option<Frame>,
    /// `position` is an animated offset from `placement`; `anchor` is the source-space pivot.
    pub transform: Transform2D,
    pub opacity: Animatable<f64>,
    pub compositing: Compositing,
    pub masks: Vec<Mask>,
    pub track_matte: Option<ResolvedMatte>,
    pub card: Option<CardStyle>,
    pub color_pipeline: Option<ResolvedColorPipeline>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectiveAudioProperties {
    pub gain: Animatable<f64>,
    pub pan: Animatable<f64>,
    pub muted: bool,
    pub normalize: bool,
    pub pitch_policy: PitchPolicy,
    pub processors: Vec<AudioProcessor>,
    pub crossfade: Option<AudioCrossfade>,
    pub sidechain: Option<ResolvedSidechain>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedMatte {
    pub relation_id: RelationId,
    pub source_clip_id: ItemId,
    pub mode: TrackMatteMode,
    pub invert: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedSidechain {
    pub relation_id: RelationId,
    pub source: SidechainSource,
    pub threshold_db: f64,
    pub ratio: f64,
    pub attack_ms: f64,
    pub release_ms: f64,
    pub active_range: Option<TimeRange>,
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{
    ApplyId, ApplyMix, ApplyStageId, EffectInstance, ItemId, RationalTime, SequenceId, TimeRange,
    TrackId, TrackMatteMode, VisualProperties,
};

use super::{ApplicationHeader, MaterialInsertion};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatteClipInsertion {
    pub material: MaterialInsertion,
    pub sequence_id: SequenceId,
    pub track_id: TrackId,
    pub clip_id: ItemId,
    pub record_range: TimeRange,
    pub source_start: RationalTime,
    pub visual: VisualProperties,
    pub before_id: Option<ItemId>,
    pub after_id: Option<ItemId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatteApplication {
    pub header: ApplicationHeader,
    pub target_clip_id: ItemId,
    pub matte: MatteClipInsertion,
    pub mode: TrackMatteMode,
    pub invert: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetouchApplication {
    pub header: ApplicationHeader,
    pub sequence_id: SequenceId,
    pub target_clip_id: ItemId,
    pub record_range: TimeRange,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matte: Option<RetouchMatteApplication>,
    pub apply_id: ApplyId,
    #[serde(default)]
    pub apply_mix: ApplyMix,
    pub time: super::ClipTimeBinding,
    pub effects: Vec<RetouchEffectApplication>,
    pub controls: Vec<RetouchControlApplication>,
    pub before_apply_id: Option<ApplyId>,
    pub after_apply_id: Option<ApplyId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetouchMatteApplication {
    pub insertion: MatteClipInsertion,
    pub mode: TrackMatteMode,
    pub invert: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetouchEffectApplication {
    pub stage_id: ApplyStageId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_range: Option<TimeRange>,
    pub effect: EffectInstance,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RetouchControlApplication {
    pub control: String,
    pub effect_id: veac_ir::EffectId,
    pub effect_parameter: String,
    pub keyframe_id_prefix: String,
}

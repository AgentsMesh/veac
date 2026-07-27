use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Animatable, ApplyId, ApplyStageId, BlendMode, ColorPipeline, EffectInstance, ItemId, Mask,
    TimeRange, TrackId,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Apply {
    pub id: ApplyId,
    pub enabled: bool,
    pub record_range: TimeRange,
    pub target: ApplyTarget,
    pub stages: Vec<ApplyStage>,
    pub mix: ApplyMix,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ApplyTarget {
    CompositeBand {
        from_track_id: TrackId,
        through_track_id: TrackId,
    },
    Layer {
        track_id: TrackId,
    },
    ItemSet {
        item_ids: Vec<ItemId>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApplyStage {
    pub id: ApplyStageId,
    pub enabled: bool,
    /// Half-open range relative to the owning Apply's record start.
    pub active_range: Option<TimeRange>,
    pub operation: ApplyOperation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ApplyOperation {
    Color { pipeline: ColorPipeline },
    Effect { effect: EffectInstance },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApplyMix {
    pub opacity: Animatable<f64>,
    pub blend_mode: BlendMode,
    pub masks: Vec<Mask>,
}

impl Default for ApplyMix {
    fn default() -> Self {
        Self {
            opacity: Animatable::constant(1.0),
            blend_mode: BlendMode::Normal,
            masks: Vec::new(),
        }
    }
}

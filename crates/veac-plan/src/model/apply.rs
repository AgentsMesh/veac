use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{ApplyId, ApplyMix, ApplyStageId, Effect, ItemId, TimeRange, TrackId};

use super::{ResolvedColorPipeline, ResolvedMatte};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedApply {
    pub id: ApplyId,
    pub source_order: u32,
    pub record_range: TimeRange,
    pub target: ResolvedApplyTarget,
    pub stages: Vec<ResolvedApplyStage>,
    pub mix: ApplyMix,
    pub matte: Option<ResolvedMatte>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedApplyTarget {
    CompositeBand {
        from_track_id: TrackId,
        through_track_id: TrackId,
        track_ids: Vec<TrackId>,
        active_ranges: Vec<TimeRange>,
    },
    Layer {
        track_id: TrackId,
        item_ids: Vec<ItemId>,
        active_ranges: Vec<TimeRange>,
    },
    ItemSet {
        items: Vec<ResolvedApplyItem>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedApplyItem {
    pub track_id: TrackId,
    pub item_id: ItemId,
    pub active_range: TimeRange,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedApplyStage {
    pub id: ApplyStageId,
    /// Absolute sequence-time gate.
    pub active_range: TimeRange,
    pub operation: ResolvedApplyOperation,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedApplyOperation {
    Color { pipeline: ResolvedColorPipeline },
    Effect { effect: Effect },
}

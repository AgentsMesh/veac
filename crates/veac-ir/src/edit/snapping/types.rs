use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SnapRequest {
    pub sequence_id: SequenceId,
    pub time: RationalTime,
    pub tolerance: RationalTime,
    pub include_clip_edges: bool,
    pub include_frame_grid: bool,
    pub excluded_items: Vec<ItemId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SnapTarget {
    ClipStart { track_id: TrackId, item_id: ItemId },
    ClipEnd { track_id: TrackId, item_id: ItemId },
    FrameGrid { frame_index: i64 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SnapCandidate {
    pub time: RationalTime,
    pub distance: RationalTime,
    pub target: SnapTarget,
}

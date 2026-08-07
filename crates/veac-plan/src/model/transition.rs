use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{ItemId, RationalTime, RelationId, TimeRange, TransitionAlignment, TransitionKind};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTransition {
    pub relation_id: RelationId,
    pub kind: TransitionKind,
    pub outgoing_clip_id: ItemId,
    pub incoming_clip_id: ItemId,
    pub alignment: TransitionAlignment,
    /// Conceptual centered cut: floor(duration / 2) ticks after the overlap starts.
    pub cut_time: RationalTime,
    /// Exact intersection of the endpoint record ranges.
    pub record_window: TimeRange,
    /// Full overlap expressed in outgoing clip-local record time.
    pub outgoing_range: TimeRange,
    /// Full overlap expressed in incoming clip-local record time.
    pub incoming_range: TimeRange,
}

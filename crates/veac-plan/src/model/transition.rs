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
    /// Exact record-timeline cut shared by the outgoing and incoming clips.
    pub cut_time: RationalTime,
    /// Exact record-timeline interval in which the transition is active.
    pub record_window: TimeRange,
    pub outgoing_handle: TransitionHandle,
    pub incoming_handle: TransitionHandle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TransitionHandle {
    /// Clip-local record-time offset at which this side of the transition begins.
    pub offset: RationalTime,
    /// Clip-local record-time duration; zero is valid for before/after-cut alignment.
    pub duration: RationalTime,
}

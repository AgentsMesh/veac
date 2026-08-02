use crate::{ItemId, RelationId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrimEdge {
    In,
    Out,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RelationFragmentId {
    pub relation_id: RelationId,
    pub right_relation_id: RelationId,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct OverwriteFragment {
    pub source_clip_id: ItemId,
    pub right_fragment_id: ItemId,
    pub relation_fragments: Vec<RelationFragmentId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct LinkedSplitFragment {
    pub source_clip_id: ItemId,
    pub right_clip_id: ItemId,
    pub relation_fragments: Vec<RelationFragmentId>,
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ChangedObjectId {
    Project { id: ProjectId },
    Sequence { id: SequenceId },
    Track { id: TrackId },
    MulticamGroup { id: MulticamGroupId },
    Material { id: MaterialId },
    Output { id: RenderConfigId },
    Item { id: ItemId },
    Apply { id: ApplyId },
    Effect { id: EffectId },
    Keyframe { id: KeyframeId },
    Annotation { id: AnnotationId },
    Relation { id: RelationId },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EditBatch {
    pub operation_id: OperationId,
    #[schemars(range(max = 9007199254740991u64))]
    pub base_revision: u64,
    pub atomic: bool,
    pub preconditions: Vec<Precondition>,
    pub operations: Vec<EditOperation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Precondition {
    ClipExists {
        clip_id: ItemId,
    },
    ClipSourceEquals {
        clip_id: ItemId,
        source: Box<ClipSource>,
    },
    TrackUnlocked {
        track_id: TrackId,
    },
    ApplyExists {
        apply_id: ApplyId,
    },
    ApplyEquals {
        apply_id: ApplyId,
        apply: Box<Apply>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum EditOutcome {
    Applied {
        project: ProjectEnvelope,
        new_revision: u64,
        changed_objects: Vec<ChangedObjectId>,
        normalized_operations: Vec<EditOperation>,
    },
    NoChange {
        project: ProjectEnvelope,
        current_revision: u64,
        operation_recorded: bool,
    },
    Conflict {
        current_revision: u64,
        diagnostics: Vec<Diagnostic>,
    },
    Rejected {
        current_revision: u64,
        diagnostics: Vec<Diagnostic>,
    },
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::*;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum StructureEdit {
    InsertMaterial {
        material: Box<Material>,
        before_id: Option<MaterialId>,
        after_id: Option<MaterialId>,
    },
    RemoveMaterial {
        material_id: MaterialId,
    },
    SetMaterial {
        material_id: MaterialId,
        material: Box<Material>,
    },
    RelinkMaterial {
        material_id: MaterialId,
        source: MaterialSource,
        identity: Option<MediaIdentity>,
        probe: Option<Box<MediaProbeSnapshot>>,
    },
    InsertSequence {
        sequence: Box<Sequence>,
        relations: Vec<Relation>,
        before_id: Option<SequenceId>,
        after_id: Option<SequenceId>,
    },
    RemoveSequence {
        sequence_id: SequenceId,
    },
    SetSequenceName {
        sequence_id: SequenceId,
        name: String,
    },
    SetSequenceSettings {
        sequence_id: SequenceId,
        settings: SequenceSettings,
    },
    InsertTrack {
        sequence_id: SequenceId,
        track: Box<Track>,
        before_id: Option<TrackId>,
        after_id: Option<TrackId>,
    },
    RemoveTrack {
        track_id: TrackId,
    },
    SetTrackState {
        track_id: TrackId,
        state: TrackState,
    },
    SetTrackKind {
        track_id: TrackId,
        kind: TrackKind,
    },
    SetTrackRouting {
        track_id: TrackId,
        routing: TrackRouting,
    },
    SetTrackOrder {
        track_id: TrackId,
        order: i32,
    },
    SetTrackPlacementMode {
        track_id: TrackId,
        placement_mode: PlacementMode,
    },
    InsertMulticamGroup {
        group: Box<MulticamGroup>,
        before_id: Option<MulticamGroupId>,
        after_id: Option<MulticamGroupId>,
    },
    RemoveMulticamGroup {
        group_id: MulticamGroupId,
    },
    InsertOutput {
        output: Box<RenderConfig>,
        before_id: Option<RenderConfigId>,
        after_id: Option<RenderConfigId>,
    },
    RemoveOutput {
        output_id: RenderConfigId,
    },
    SetOutput {
        output_id: RenderConfigId,
        output: Box<RenderConfig>,
    },
    SetEntrySequence {
        sequence_id: SequenceId,
    },
    InsertRelation {
        relation: Relation,
    },
    SetRelation {
        relation: Relation,
    },
    RemoveRelation {
        relation_id: RelationId,
    },
    InsertApply {
        sequence_id: SequenceId,
        apply: Box<Apply>,
        before_id: Option<ApplyId>,
        after_id: Option<ApplyId>,
    },
    SetApply {
        apply_id: ApplyId,
        apply: Box<Apply>,
    },
    RemoveApply {
        apply_id: ApplyId,
    },
    MoveApply {
        apply_id: ApplyId,
        before_id: Option<ApplyId>,
        after_id: Option<ApplyId>,
    },
}

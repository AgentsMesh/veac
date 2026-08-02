use veac_ir::*;

const FIXTURE: &str = include_str!("fixtures/minimal-project.json");

fn relation() -> Relation {
    Relation {
        id: RelationId::new("rel_public_link").unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::AvLink {
            video: RelationEndpoint::Item {
                item_id: ItemId::new("itm_video").unwrap(),
            },
            audio: vec![RelationEndpoint::Item {
                item_id: ItemId::new("itm_audio").unwrap(),
            }],
        },
    }
}

fn project_with_audio() -> ProjectEnvelope {
    let mut project = decode_canonical_json(FIXTURE).unwrap();
    let sequence = &mut project.project.sequences[0];
    sequence.tracks[0].placement_mode = PlacementMode::Free;
    let mut audio_clip = sequence.tracks[0].clips[0].clone();
    audio_clip.id = ItemId::new("itm_audio").unwrap();
    sequence.tracks.push(Track {
        id: TrackId::new("trk_audio").unwrap(),
        kind: TrackKind::Audio,
        order: 1,
        placement_mode: PlacementMode::Free,
        state: TrackState {
            enabled: true,
            muted: false,
            solo: false,
            locked: false,
        },
        routing: TrackRouting::Default,
        clips: vec![audio_clip],
    });
    project
}

fn batch(project: &ProjectEnvelope, id: &str, operation: EditOperation) -> EditBatch {
    EditBatch {
        operation_id: OperationId::new(id).unwrap(),
        base_revision: project.project.revision,
        atomic: true,
        preconditions: vec![],
        operations: vec![operation],
    }
}

#[test]
fn public_edit_protocol_inserts_and_uses_a_canonical_av_link() {
    let project = project_with_audio();
    let insert = batch(
        &project,
        "op_public_link",
        EditOperation::EditStructure {
            edit: StructureEdit::InsertRelation {
                relation: relation(),
            },
        },
    );
    let linked = applied(apply_edit_batch(&project, &insert));
    let movement = batch(
        &linked,
        "op_public_link_move",
        EditOperation::MoveClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            record_start: RationalTime::new(120, 600).unwrap(),
        },
    );
    let moved = applied(apply_edit_batch(&linked, &movement));
    assert_eq!(
        moved.project.sequences[0].tracks[0].clips[0]
            .record_range
            .start,
        RationalTime::new(120, 600).unwrap()
    );
    assert_eq!(
        moved.project.sequences[0].tracks[1].clips[0]
            .record_range
            .start,
        RationalTime::new(120, 600).unwrap()
    );
    let json = canonical_json(&moved).unwrap();
    assert!(json.contains("rel_public_link"));
    assert!(!json.contains("\"av_links\""));
}

#[test]
fn public_edit_protocol_removes_a_canonical_av_link() {
    let mut project = project_with_audio();
    project.project.relations.push(relation());
    let remove = batch(
        &project,
        "op_public_unlink",
        EditOperation::EditStructure {
            edit: StructureEdit::RemoveRelation {
                relation_id: RelationId::new("rel_public_link").unwrap(),
            },
        },
    );
    let unlinked = applied(apply_edit_batch(&project, &remove));
    assert!(unlinked.project.relations.is_empty());
}

#[test]
fn relation_insert_honors_locked_endpoints_atomically() {
    let mut project = project_with_audio();
    project.project.sequences[0].tracks[1].state.locked = true;
    let insert = batch(
        &project,
        "op_public_locked_link",
        EditOperation::EditStructure {
            edit: StructureEdit::InsertRelation {
                relation: relation(),
            },
        },
    );
    assert!(matches!(
        apply_edit_batch(&project, &insert),
        EditOutcome::Rejected { .. }
    ));
    assert!(project.project.relations.is_empty());
}

fn applied(outcome: EditOutcome) -> ProjectEnvelope {
    match outcome {
        EditOutcome::Applied { project, .. } => project,
        EditOutcome::Rejected { diagnostics, .. } => {
            panic!("expected applied edit, got {diagnostics:?}")
        }
        other => panic!("expected applied edit, got {other:?}"),
    }
}

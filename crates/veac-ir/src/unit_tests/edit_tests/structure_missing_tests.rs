use super::*;
use crate::test_support::multicam_project;

#[test]
fn every_top_level_remove_reports_its_missing_typed_object() {
    let project = sample_project();
    let cases = [
        (
            StructureEdit::RemoveMaterial {
                material_id: MaterialId::new("med_missing").unwrap(),
            },
            "material does not exist",
        ),
        (
            StructureEdit::RemoveSequence {
                sequence_id: SequenceId::new("seq_missing").unwrap(),
            },
            "sequence does not exist",
        ),
        (
            StructureEdit::RemoveTrack {
                track_id: TrackId::new("trk_missing").unwrap(),
            },
            "track does not exist",
        ),
        (
            StructureEdit::RemoveOutput {
                output_id: RenderConfigId::new("out_missing").unwrap(),
            },
            "output does not exist",
        ),
        (
            StructureEdit::RemoveMulticamGroup {
                group_id: MulticamGroupId::new("mcg_missing").unwrap(),
            },
            "multicam group does not exist",
        ),
    ];
    for (index, (edit, message)) in cases.into_iter().enumerate() {
        let outcome = apply_edit_batch(
            &project,
            &batch(
                &format!("op_remove_missing_{index}"),
                &project,
                vec![EditOperation::EditStructure { edit }],
            ),
        );
        let EditOutcome::Rejected { diagnostics, .. } = outcome else {
            panic!("missing structure object must reject")
        };
        assert!(diagnostics.iter().any(|value| value.message == message));
    }
}

#[test]
fn missing_insert_set_and_timing_targets_reject_atomically() {
    let project = sample_project();
    let mut track = project.project.sequences[0].tracks[1].clone();
    track.id = TrackId::new("trk_new").unwrap();
    let mut output = project.project.render_configs[0].clone();
    output.id = RenderConfigId::new("out_missing").unwrap();
    let failures = vec![
        EditOperation::RippleInsert {
            sequence_id: SequenceId::new("seq_missing").unwrap(),
            track_id: TrackId::new("trk_missing").unwrap(),
            clip: Box::new(generated_clip("itm_ripple_missing", 0)),
        },
        EditOperation::EditStructure {
            edit: StructureEdit::InsertTrack {
                sequence_id: SequenceId::new("seq_missing").unwrap(),
                track: Box::new(track),
                before_id: None,
                after_id: None,
            },
        },
        EditOperation::EditStructure {
            edit: StructureEdit::InsertRelation {
                relation: Relation {
                    id: RelationId::new("rel_new").unwrap(),
                    sequence_id: SequenceId::new("seq_missing").unwrap(),
                    kind: RelationKind::Group {
                        members: vec![RelationEndpoint::Item {
                            item_id: ItemId::new("itm_video").unwrap(),
                        }],
                    },
                },
            },
        },
        EditOperation::EditStructure {
            edit: StructureEdit::SetOutput {
                output_id: output.id.clone(),
                output: Box::new(output),
            },
        },
        EditOperation::SlipClip {
            clip_id: ItemId::new("itm_missing").unwrap(),
            source_delta: time(1),
        },
        EditOperation::SetMulticamSwitches {
            clip_id: ItemId::new("itm_video").unwrap(),
            switches: vec![],
        },
    ];
    for (index, operation) in failures.into_iter().enumerate() {
        let edit = batch(
            &format!("op_missing_target_{index}"),
            &project,
            vec![operation],
        );
        assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    }

    let multicam = multicam_project();
    let group = &multicam.project.multicam_groups[0];
    let edit = batch(
        "op_missing_multicam_group",
        &multicam,
        vec![EditOperation::SetMulticamGroup {
            group_id: MulticamGroupId::new("mcg_missing").unwrap(),
            sync: group.sync.clone(),
            angles: group.angles.clone(),
        }],
    );
    assert_rejected(apply_edit_batch(&multicam, &edit), "EDIT_REJECTED");
}

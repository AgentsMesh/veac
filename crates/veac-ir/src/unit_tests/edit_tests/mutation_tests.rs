use crate::test_support::time;

use super::*;

#[test]
fn remove_move_replace_and_text_edits_have_typed_failures() {
    let project = sample_project();
    let successes = [
        vec![EditOperation::RemoveClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
        }],
        vec![EditOperation::MoveClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
            record_start: time(60),
        }],
        vec![
            EditOperation::ReplaceSource {
                clip_id: ItemId::new("itm_video").unwrap(),
                source: Box::new(ClipSource::FreezeFrame {
                    material_id: MaterialId::new("med_video").unwrap(),
                    source_time: time(100),
                }),
                source_mapping: None,
            },
            EditOperation::SetAudio {
                clip_id: ItemId::new("itm_video").unwrap(),
                audio: None,
            },
        ],
    ];
    for (index, operations) in successes.into_iter().enumerate() {
        let edit = batch(&format!("op_success_{index}"), &project, operations);
        assert!(matches!(
            apply_edit_batch(&project, &edit),
            EditOutcome::Applied { .. }
        ));
    }

    let failures = [
        EditOperation::RemoveClip {
            clip_id: ItemId::new("itm_missing").unwrap(),
        },
        EditOperation::MoveClip {
            clip_id: ItemId::new("itm_missing").unwrap(),
            record_start: time(0),
        },
        EditOperation::ReplaceSource {
            clip_id: ItemId::new("itm_missing").unwrap(),
            source: Box::new(ClipSource::Generated {
                generator: Generator::Silence,
            }),
            source_mapping: None,
        },
        EditOperation::SetText {
            clip_id: ItemId::new("itm_video").unwrap(),
            text: "not text".to_owned(),
        },
        EditOperation::SetClipEnabled {
            clip_id: ItemId::new("itm_missing").unwrap(),
            enabled: false,
        },
    ];
    for (index, operation) in failures.into_iter().enumerate() {
        let edit = batch(&format!("op_missing_{index}"), &project, vec![operation]);
        assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    }
}

#[test]
fn locked_tracks_reject_every_mutation() {
    let mut project = sample_project();
    project.project.sequences[0].tracks[1].state.locked = true;
    let operations = [
        EditOperation::InsertClip {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_captions").unwrap(),
            clip: Box::new(generated_clip("itm_locked_insert", 300)),
            before_id: None,
            after_id: None,
        },
        EditOperation::RemoveClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
        },
        EditOperation::MoveClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
            record_start: time(10),
        },
        EditOperation::ReplaceSource {
            clip_id: ItemId::new("itm_caption").unwrap(),
            source: Box::new(ClipSource::Generated {
                generator: Generator::Transparent,
            }),
            source_mapping: None,
        },
        EditOperation::SetText {
            clip_id: ItemId::new("itm_caption").unwrap(),
            text: "locked".to_owned(),
        },
        EditOperation::SetClipEnabled {
            clip_id: ItemId::new("itm_caption").unwrap(),
            enabled: false,
        },
    ];
    for (index, operation) in operations.into_iter().enumerate() {
        let edit = batch(&format!("op_locked_{index}"), &project, vec![operation]);
        assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    }
}

#[test]
fn cancelling_operations_report_no_change() {
    let project = sample_project();
    let edit = batch(
        "op_cancel",
        &project,
        vec![
            EditOperation::SetClipEnabled {
                clip_id: ItemId::new("itm_caption").unwrap(),
                enabled: false,
            },
            EditOperation::SetClipEnabled {
                clip_id: ItemId::new("itm_caption").unwrap(),
                enabled: true,
            },
        ],
    );
    assert!(matches!(
        apply_edit_batch(&project, &edit),
        EditOutcome::NoChange { .. }
    ));
}

use crate::test_support::time;

use super::*;

#[test]
fn insert_uses_stable_neighbors() {
    let project = sample_project();
    let after = batch(
        "op_insert_after",
        &project,
        vec![EditOperation::InsertClip {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_captions").unwrap(),
            clip: Box::new(generated_clip("itm_after", 300)),
            before_id: None,
            after_id: Some(ItemId::new("itm_caption").unwrap()),
        }],
    );
    let inserted = applied(apply_edit_batch(&project, &after));
    assert_eq!(
        inserted.project.sequences[0].tracks[1].clips[1].id.as_str(),
        "itm_after"
    );

    let reorder = batch(
        "op_reorder",
        &inserted,
        vec![EditOperation::MoveClip {
            clip_id: ItemId::new("itm_after").unwrap(),
            record_start: time(0),
        }],
    );
    assert!(matches!(
        apply_edit_batch(&inserted, &reorder),
        EditOutcome::Applied { .. }
    ));

    for (id, before, after) in [
        ("itm_append", None, None),
        (
            "itm_before",
            Some(ItemId::new("itm_caption").unwrap()),
            None,
        ),
    ] {
        let edit = batch(
            &format!("op_{id}"),
            &project,
            vec![EditOperation::InsertClip {
                sequence_id: SequenceId::new("seq_main").unwrap(),
                track_id: TrackId::new("trk_captions").unwrap(),
                clip: Box::new(generated_clip(id, if before.is_some() { 0 } else { 300 })),
                before_id: before,
                after_id: after,
            }],
        );
        assert!(matches!(
            apply_edit_batch(&project, &edit),
            EditOutcome::Applied { .. }
        ));
    }
}

#[test]
fn insert_rejects_ambiguous_duplicate_and_missing_targets() {
    let project = sample_project();
    let operations = [
        EditOperation::InsertClip {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_captions").unwrap(),
            clip: Box::new(generated_clip("itm_both", 0)),
            before_id: Some(ItemId::new("itm_caption").unwrap()),
            after_id: Some(ItemId::new("itm_caption").unwrap()),
        },
        EditOperation::InsertClip {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_captions").unwrap(),
            clip: Box::new(generated_clip("itm_missing_neighbor", 0)),
            before_id: Some(ItemId::new("itm_missing").unwrap()),
            after_id: None,
        },
        EditOperation::InsertClip {
            sequence_id: SequenceId::new("seq_missing").unwrap(),
            track_id: TrackId::new("trk_missing").unwrap(),
            clip: Box::new(generated_clip("itm_missing_track", 0)),
            before_id: None,
            after_id: None,
        },
    ];
    for (index, operation) in operations.into_iter().enumerate() {
        let edit = batch(&format!("op_bad_insert_{index}"), &project, vec![operation]);
        assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    }
    let duplicate = batch(
        "op_duplicate",
        &project,
        vec![EditOperation::InsertClip {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_captions").unwrap(),
            clip: Box::new(project.project.sequences[0].tracks[1].clips[0].clone()),
            before_id: None,
            after_id: None,
        }],
    );
    assert_rejected(apply_edit_batch(&project, &duplicate), "EDIT_REJECTED");
}

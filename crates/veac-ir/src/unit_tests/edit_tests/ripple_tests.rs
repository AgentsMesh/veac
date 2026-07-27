use crate::test_support::time;

use super::*;

#[test]
fn ripple_insert_and_delete_shift_only_following_clips() {
    let project = magnetic_project();
    let mut inserted_clip = project.project.sequences[0].tracks[0].clips[0].clone();
    inserted_clip.id = ItemId::new("itm_inserted").unwrap();
    inserted_clip.record_range = range(600, 300);
    set_linear_start(&mut inserted_clip, time(0));
    let insert = batch(
        "op_ripple_insert",
        &project,
        vec![EditOperation::RippleInsert {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_video").unwrap(),
            clip: Box::new(inserted_clip),
        }],
    );
    let inserted = applied(apply_edit_batch(&project, &insert));
    let clips = &inserted.project.sequences[0].tracks[0].clips;
    assert_eq!(clips[0].record_range, range(0, 600));
    assert_eq!(clips[1].id.as_str(), "itm_inserted");
    assert_eq!(clips[2].record_range.start, time(900));
    assert_eq!(clips[3].record_range.start, time(1500));

    let delete = batch(
        "op_ripple_delete",
        &inserted,
        vec![EditOperation::RippleDelete {
            clip_id: ItemId::new("itm_inserted").unwrap(),
        }],
    );
    let restored = applied(apply_edit_batch(&inserted, &delete));
    let clips = &restored.project.sequences[0].tracks[0].clips;
    assert_eq!(clips.len(), 3);
    assert_eq!(clips[1].record_range.start, time(600));
    assert_eq!(clips[2].record_range.start, time(1200));
}

#[test]
fn roll_moves_one_cut_without_changing_outer_range() {
    let project = magnetic_project();
    let edit = batch(
        "op_roll",
        &project,
        vec![EditOperation::RollEdit {
            left_clip_id: ItemId::new("itm_video").unwrap(),
            right_clip_id: ItemId::new("itm_middle").unwrap(),
            delta: time(60),
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let clips = &result.project.sequences[0].tracks[0].clips;
    assert_eq!(clips[0].record_range, range(0, 660));
    assert_eq!(clips[1].record_range, range(660, 540));
    assert_eq!(linear_start(&clips[1]), time(660));
    assert_eq!(clips[1].record_range.end().unwrap(), time(1200));
}

#[test]
fn slide_preserves_target_duration_and_neighbor_outer_edges() {
    let project = magnetic_project();
    let edit = batch(
        "op_slide",
        &project,
        vec![EditOperation::SlideClip {
            clip_id: ItemId::new("itm_middle").unwrap(),
            delta: time(60),
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let clips = &result.project.sequences[0].tracks[0].clips;
    assert_eq!(clips[0].record_range, range(0, 660));
    assert_eq!(clips[1].record_range, range(660, 600));
    assert_eq!(clips[2].record_range, range(1260, 540));
    assert_eq!(linear_start(&clips[2]), time(1260));
    assert_eq!(clips[2].record_range.end().unwrap(), time(1800));
}

#[test]
fn adjacent_edits_reject_missing_handles_and_nonpositive_results() {
    let project = magnetic_project();
    let failures = [
        EditOperation::RollEdit {
            left_clip_id: ItemId::new("itm_video").unwrap(),
            right_clip_id: ItemId::new("itm_right").unwrap(),
            delta: time(1),
        },
        EditOperation::RollEdit {
            left_clip_id: ItemId::new("itm_video").unwrap(),
            right_clip_id: ItemId::new("itm_middle").unwrap(),
            delta: time(600),
        },
        EditOperation::SlideClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            delta: time(1),
        },
        EditOperation::SlideClip {
            clip_id: ItemId::new("itm_middle").unwrap(),
            delta: time(600),
        },
    ];
    for (index, operation) in failures.into_iter().enumerate() {
        let edit = batch(
            &format!("op_bad_adjacent_{index}"),
            &project,
            vec![operation],
        );
        assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    }
}

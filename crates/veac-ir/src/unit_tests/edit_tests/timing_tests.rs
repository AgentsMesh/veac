use crate::test_support::time;

use super::*;

#[test]
fn split_preserves_record_and_source_continuity() {
    let project = sample_project();
    let edit = batch(
        "op_split",
        &project,
        vec![EditOperation::SplitClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            at: time(300),
            right_clip_id: ItemId::new("itm_split_right").unwrap(),
            relation_fragments: vec![],
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let clips = &result.project.sequences[0].tracks[0].clips;
    assert_eq!(clips.len(), 2);
    assert_eq!(clips[0].record_range, range(0, 300));
    assert_eq!(clips[1].record_range, range(300, 300));
    assert_eq!(linear_start(&clips[1]), time(300));
    assert!(crate::test_support::transition_from(&result, "itm_video").is_none());
}

#[test]
fn split_and_slip_reject_invalid_targets_without_mutation() {
    let project = sample_project();
    for (index, at) in [time(0), time(600), time(700)].into_iter().enumerate() {
        let edit = batch(
            &format!("op_split_boundary_{index}"),
            &project,
            vec![EditOperation::SplitClip {
                clip_id: ItemId::new("itm_video").unwrap(),
                at,
                right_clip_id: ItemId::new(format!("itm_bad_{index}")).unwrap(),
                relation_fragments: vec![],
            }],
        );
        assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
    }
    let duplicate = batch(
        "op_split_duplicate",
        &project,
        vec![EditOperation::SplitClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            at: time(300),
            right_clip_id: ItemId::new("itm_caption").unwrap(),
            relation_fragments: vec![],
        }],
    );
    assert_rejected(apply_edit_batch(&project, &duplicate), "EDIT_REJECTED");

    let slip_caption = batch(
        "op_slip_caption",
        &project,
        vec![EditOperation::SlipClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
            source_delta: time(10),
        }],
    );
    assert_rejected(apply_edit_batch(&project, &slip_caption), "EDIT_REJECTED");
}

#[test]
fn ripple_trim_and_slip_update_exact_source_time() {
    let project = magnetic_project();
    let trim = batch(
        "op_ripple_trim_in",
        &project,
        vec![EditOperation::TrimClip {
            clip_id: ItemId::new("itm_middle").unwrap(),
            edge: TrimEdge::In,
            delta: time(60),
            ripple: true,
        }],
    );
    let trimmed = applied(apply_edit_batch(&project, &trim));
    let clips = &trimmed.project.sequences[0].tracks[0].clips;
    assert_eq!(clips[1].record_range, range(600, 540));
    assert_eq!(linear_start(&clips[1]), time(660));
    assert_eq!(clips[2].record_range.start, time(1140));

    let slip = batch(
        "op_slip",
        &project,
        vec![EditOperation::SlipClip {
            clip_id: ItemId::new("itm_middle").unwrap(),
            source_delta: time(45),
        }],
    );
    let slipped = applied(apply_edit_batch(&project, &slip));
    let clip = &slipped.project.sequences[0].tracks[0].clips[1];
    assert_eq!(clip.record_range, range(600, 600));
    assert_eq!(linear_start(clip), time(645));
}

#[test]
fn trim_out_ripples_and_free_track_trim_in_moves_only_the_edge() {
    let project = magnetic_project();
    let trim_out = batch(
        "op_ripple_trim_out",
        &project,
        vec![EditOperation::TrimClip {
            clip_id: ItemId::new("itm_middle").unwrap(),
            edge: TrimEdge::Out,
            delta: time(60),
            ripple: true,
        }],
    );
    let result = applied(apply_edit_batch(&project, &trim_out));
    let clips = &result.project.sequences[0].tracks[0].clips;
    assert_eq!(clips[1].record_range, range(600, 660));
    assert_eq!(clips[2].record_range.start, time(1260));

    let free = sample_project();
    let trim_in = batch(
        "op_free_trim_in",
        &free,
        vec![EditOperation::TrimClip {
            clip_id: ItemId::new("itm_caption").unwrap(),
            edge: TrimEdge::In,
            delta: time(60),
            ripple: false,
        }],
    );
    let result = applied(apply_edit_batch(&free, &trim_in));
    assert_eq!(
        result.project.sequences[0].tracks[1].clips[0].record_range,
        range(60, 240)
    );
}

#[test]
fn trim_rejects_inexact_source_time_at_the_project_timebase() {
    let mut project = magnetic_project();
    *linear_map_mut(&mut project.project.sequences[0].tracks[0].clips[1]).1 =
        Rational::new(1, 2).unwrap();
    let edit = batch(
        "op_inexact_trim",
        &project,
        vec![EditOperation::TrimClip {
            clip_id: ItemId::new("itm_middle").unwrap(),
            edge: TrimEdge::In,
            delta: time(1),
            ripple: true,
        }],
    );
    assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
}

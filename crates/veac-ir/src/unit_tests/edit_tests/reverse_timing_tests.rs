use super::*;

#[test]
fn reverse_split_keeps_the_high_source_range_on_the_left_and_low_range_on_the_right() {
    let mut project = sample_project();
    let (start, _, _, direction) =
        linear_map_mut(&mut project.project.sequences[0].tracks[0].clips[0]);
    *direction = PlaybackDirection::Reverse;
    *start = time(100);
    let edit = batch(
        "op_reverse_split",
        &project,
        vec![EditOperation::SplitClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            at: time(300),
            right_clip_id: ItemId::new("itm_reverse_right").unwrap(),
            relation_fragments: vec![],
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let clips = &result.project.sequences[0].tracks[0].clips;
    assert_eq!(linear_start(&clips[0]), time(400));
    assert_eq!(linear_start(&clips[1]), time(100));
}

#[test]
fn reverse_trim_in_preserves_low_bound_while_trim_out_moves_it() {
    let mut project = sample_project();
    let (start, _, _, direction) =
        linear_map_mut(&mut project.project.sequences[0].tracks[0].clips[0]);
    *direction = PlaybackDirection::Reverse;
    *start = time(100);
    let trim_in = batch(
        "op_reverse_trim_in",
        &project,
        vec![EditOperation::TrimClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            edge: TrimEdge::In,
            delta: time(60),
            ripple: true,
        }],
    );
    let result = applied(apply_edit_batch(&project, &trim_in));
    assert_eq!(
        linear_start(&result.project.sequences[0].tracks[0].clips[0]),
        time(100)
    );
    let trim_out = batch(
        "op_reverse_trim_out",
        &project,
        vec![EditOperation::TrimClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            edge: TrimEdge::Out,
            delta: time(-60),
            ripple: true,
        }],
    );
    let result = applied(apply_edit_batch(&project, &trim_out));
    assert_eq!(
        linear_start(&result.project.sequences[0].tracks[0].clips[0]),
        time(160)
    );
}

#[test]
fn reverse_roll_and_slide_adjust_only_out_edges() {
    let mut project = magnetic_project();
    for clip in &mut project.project.sequences[0].tracks[0].clips {
        *linear_map_mut(clip).3 = PlaybackDirection::Reverse;
    }
    set_linear_start(
        &mut project.project.sequences[0].tracks[0].clips[0],
        time(100),
    );
    let roll = batch(
        "op_reverse_roll",
        &project,
        vec![EditOperation::RollEdit {
            left_clip_id: ItemId::new("itm_video").unwrap(),
            right_clip_id: ItemId::new("itm_middle").unwrap(),
            delta: time(60),
        }],
    );
    let result = applied(apply_edit_batch(&project, &roll));
    let clips = &result.project.sequences[0].tracks[0].clips;
    assert_eq!(linear_start(&clips[0]), time(40));
    assert_eq!(linear_start(&clips[1]), time(600));

    let slide = batch(
        "op_reverse_slide",
        &project,
        vec![EditOperation::SlideClip {
            clip_id: ItemId::new("itm_middle").unwrap(),
            delta: time(60),
        }],
    );
    let result = applied(apply_edit_batch(&project, &slide));
    let clips = &result.project.sequences[0].tracks[0].clips;
    assert_eq!(linear_start(&clips[0]), time(40));
    assert_eq!(linear_start(&clips[2]), time(1200));
}

#[test]
fn partial_timing_edits_of_looped_media_fail_closed() {
    let mut project = sample_project();
    *linear_map_mut(&mut project.project.sequences[0].tracks[0].clips[0]).2 = 2;
    let edit = batch(
        "op_split_looped",
        &project,
        vec![EditOperation::SplitClip {
            clip_id: ItemId::new("itm_video").unwrap(),
            at: time(300),
            right_clip_id: ItemId::new("itm_loop_right").unwrap(),
            relation_fragments: vec![],
        }],
    );
    assert_rejected(apply_edit_batch(&project, &edit), "EDIT_REJECTED");
}

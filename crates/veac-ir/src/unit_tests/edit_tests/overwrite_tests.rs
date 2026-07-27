use std::collections::BTreeMap;

use crate::test_support::time;

use super::*;

#[test]
fn overwrite_carves_every_overlap_shape_with_stable_fragment_ids() {
    let mut project = sample_project();
    let track = &mut project.project.sequences[0].tracks[1];
    track.clips = vec![
        generated_clip("itm_span", 0),
        generated_clip("itm_left_overlap", 50),
        generated_clip("itm_covered", 220),
        generated_clip("itm_right_overlap", 350),
    ];
    track.clips[0].record_range.duration = time(600);
    track.clips[1].record_range.duration = time(200);
    track.clips[2].record_range.duration = time(80);
    track.clips[3].record_range.duration = time(200);
    let mut replacement = generated_clip("itm_replacement", 200);
    replacement.record_range.duration = time(200);
    let edit = batch(
        "op_overwrite_shapes",
        &project,
        vec![EditOperation::OverwriteClip {
            sequence_id: SequenceId::new("seq_main").unwrap(),
            track_id: TrackId::new("trk_captions").unwrap(),
            clip: Box::new(replacement),
            split_fragments: vec![OverwriteFragment {
                source_clip_id: ItemId::new("itm_span").unwrap(),
                right_fragment_id: ItemId::new("itm_span_right").unwrap(),
                relation_fragments: vec![],
            }],
        }],
    );
    let result = applied(apply_edit_batch(&project, &edit));
    let ranges: BTreeMap<_, _> = result.project.sequences[0].tracks[1]
        .clips
        .iter()
        .map(|clip| (clip.id.as_str(), clip.record_range))
        .collect();
    assert_eq!(ranges.len(), 5);
    assert_eq!(ranges["itm_span"], range(0, 200));
    assert_eq!(ranges["itm_left_overlap"], range(50, 150));
    assert_eq!(ranges["itm_replacement"], range(200, 200));
    assert_eq!(ranges["itm_span_right"], range(400, 200));
    assert_eq!(ranges["itm_right_overlap"], range(400, 150));
    assert!(!ranges.contains_key("itm_covered"));
}

#[test]
fn overwrite_requires_exact_new_fragment_mapping_and_unlocked_track() {
    let mut project = sample_project();
    let mut replacement = project.project.sequences[0].tracks[0].clips[0].clone();
    replacement.id = ItemId::new("itm_overwrite").unwrap();
    replacement.record_range = range(200, 200);
    replacement.effects.clear();
    replacement.visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
    let operation = |fragments| EditOperation::OverwriteClip {
        sequence_id: SequenceId::new("seq_main").unwrap(),
        track_id: TrackId::new("trk_video").unwrap(),
        clip: Box::new(replacement.clone()),
        split_fragments: fragments,
    };
    let missing = batch("op_overwrite_missing", &project, vec![operation(vec![])]);
    assert_rejected(apply_edit_batch(&project, &missing), "EDIT_REJECTED");
    let extra = batch(
        "op_overwrite_extra",
        &project,
        vec![operation(vec![OverwriteFragment {
            source_clip_id: ItemId::new("itm_caption").unwrap(),
            right_fragment_id: ItemId::new("itm_fragment").unwrap(),
            relation_fragments: vec![],
        }])],
    );
    assert_rejected(apply_edit_batch(&project, &extra), "EDIT_REJECTED");
    project.project.sequences[0].tracks[0].state.locked = true;
    let locked = batch(
        "op_overwrite_locked",
        &project,
        vec![operation(vec![OverwriteFragment {
            source_clip_id: ItemId::new("itm_video").unwrap(),
            right_fragment_id: ItemId::new("itm_fragment").unwrap(),
            relation_fragments: vec![],
        }])],
    );
    assert_rejected(apply_edit_batch(&project, &locked), "EDIT_REJECTED");
}

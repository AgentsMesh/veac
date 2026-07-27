use crate::edit::ChangeSet;
use crate::test_support::{sample_project, time};

use super::*;

#[test]
fn timing_helpers_cover_lookup_lock_duration_and_negation_edges() {
    let mut project = sample_project().project;
    let video = ItemId::new("itm_video").unwrap();
    let missing = ItemId::new("itm_missing").unwrap();
    assert_eq!(
        clip_index(&project.sequences[0].tracks[0], &video).unwrap(),
        0
    );
    assert!(clip_index(&project.sequences[0].tracks[0], &missing).is_err());
    assert_eq!(
        unlocked_track(&mut project, &video).unwrap().id.as_str(),
        "trk_video"
    );
    assert!(unlocked_track(&mut project, &missing).is_err());
    project.sequences[0].tracks[0].state.locked = true;
    assert!(unlocked_track(&mut project, &video).is_err());

    assert!(require_positive(time(1), &video).is_ok());
    assert!(require_positive(time(0), &video).is_err());
    assert!(require_positive(time(-1), &video).is_err());
    assert_eq!(negative(time(5), &video).unwrap(), time(-5));
    let invalid = RationalTime {
        value: 1,
        timescale: 0,
    };
    assert!(negative(invalid, &video).is_err());
}

#[test]
fn start_order_is_total_even_for_invalid_times() {
    let mut left = sample_project().project.sequences[0].tracks[0].clips[0].clone();
    let mut right = left.clone();
    right.record_range.start = time(1);
    assert_eq!(start_order(&left, &right), Ordering::Less);
    assert_eq!(start_order(&right, &left), Ordering::Greater);
    right.record_range.start = left.record_range.start;
    assert_eq!(start_order(&left, &right), Ordering::Equal);
    left.record_range.start.timescale = 0;
    assert_eq!(start_order(&left, &right), Ordering::Equal);
}

#[test]
fn moving_to_the_current_start_is_a_true_noop() {
    let mut project = sample_project().project;
    let video = ItemId::new("itm_video").unwrap();
    let mut changed = ChangeSet::new();
    move_clip(&mut project, &video, time(0), &mut changed).unwrap();
    assert!(changed.is_empty());

    move_clip(&mut project, &video, time(10), &mut changed).unwrap();
    assert_eq!(
        project.sequences[0].tracks[0].clips[0].record_range.start,
        time(10)
    );
    assert_eq!(changed.len(), 2);
    assert!(move_clip(
        &mut project,
        &ItemId::new("itm_missing").unwrap(),
        time(0),
        &mut ChangeSet::new(),
    )
    .is_err());
}

#[test]
fn rejects_slip_without_mapping_missing_track_and_invalid_negative_time() {
    let mut project = sample_project().project;
    let item_id = project.sequences[0].tracks[0].clips[0].id.clone();
    project.sequences[0].tracks[0].clips[0].source_mapping = None;
    let mut changed = ChangeSet::new();
    let error = slip(&mut project, &item_id, time(1), &mut changed).unwrap_err();
    assert_eq!(error.code, "EDIT_REJECTED");
}

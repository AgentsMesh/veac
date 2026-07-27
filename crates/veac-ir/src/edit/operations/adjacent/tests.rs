use crate::edit::ChangeSet;
use crate::test_support::{sample_project, time};

use super::*;

#[test]
fn negative_roll_and_slide_preserve_outer_edges_and_source_continuity() {
    let project = three_clips();
    let rolled = applied(apply_edit_batch(
        &project,
        &batch(
            &project,
            "op_roll_negative",
            EditOperation::RollEdit {
                left_clip_id: id("itm_video"),
                right_clip_id: id("itm_middle"),
                delta: time(-60),
            },
        ),
    ));
    let clips = &rolled.project.sequences[0].tracks[0].clips;
    assert_eq!(clips[0].record_range.duration, time(540));
    assert_eq!(
        clips[1].record_range,
        TimeRange {
            start: time(540),
            duration: time(660)
        }
    );
    assert_eq!(source_start(&clips[1]), time(540));

    let slid = applied(apply_edit_batch(
        &project,
        &batch(
            &project,
            "op_slide_negative",
            EditOperation::SlideClip {
                clip_id: id("itm_middle"),
                delta: time(-60),
            },
        ),
    ));
    let clips = &slid.project.sequences[0].tracks[0].clips;
    assert_eq!(clips[0].record_range.duration, time(540));
    assert_eq!(clips[1].record_range.start, time(540));
    assert_eq!(
        clips[2].record_range,
        TimeRange {
            start: time(1140),
            duration: time(660)
        }
    );
    assert_eq!(source_start(&clips[2]), time(1140));
}

#[test]
fn adjacent_edits_reject_gaps_other_tracks_and_locked_tracks() {
    let mut project = three_clips().project;
    let mut changed = ChangeSet::new();
    assert!(roll(
        &mut project,
        &id("itm_video"),
        &id("itm_caption"),
        time(1),
        &mut changed,
    )
    .is_err());
    assert!(changed.is_empty());

    project.sequences[0].tracks[0].placement_mode = PlacementMode::Free;
    project.sequences[0].tracks[0].clips[1].record_range.start = time(601);
    assert!(roll(
        &mut project,
        &id("itm_video"),
        &id("itm_middle"),
        time(1),
        &mut changed,
    )
    .is_err());
    assert!(slide(&mut project, &id("itm_middle"), time(1), &mut changed).is_err());

    project.sequences[0].tracks[0].state.locked = true;
    assert!(roll(
        &mut project,
        &id("itm_video"),
        &id("itm_middle"),
        time(1),
        &mut changed,
    )
    .is_err());
    assert!(slide(&mut project, &id("itm_middle"), time(1), &mut changed).is_err());
}

#[test]
fn roll_without_a_source_handle_is_rejected_atomically() {
    let mut project = three_clips();
    set_source_start(
        &mut project.project.sequences[0].tracks[0].clips[1],
        time(0),
    );
    let edit = batch(
        &project,
        "op_roll_before_media",
        EditOperation::RollEdit {
            left_clip_id: id("itm_video"),
            right_clip_id: id("itm_middle"),
            delta: time(-1),
        },
    );
    let EditOutcome::Rejected { diagnostics, .. } = apply_edit_batch(&project, &edit) else {
        panic!("missing handle must reject")
    };
    assert!(diagnostics[0].message.contains("start of media"));
    assert_eq!(
        project.project.sequences[0].tracks[0].clips[1]
            .record_range
            .start,
        time(600)
    );
}

fn three_clips() -> ProjectEnvelope {
    let mut project = sample_project();
    let track = &mut project.project.sequences[0].tracks[0];
    track.clips[0].effects.clear();
    track.clips[0].visual.as_mut().unwrap().opacity = Animatable::constant(1.0);
    let mut middle = track.clips[0].clone();
    middle.id = id("itm_middle");
    middle.record_range.start = time(600);
    set_source_start(&mut middle, time(600));
    let mut right = middle.clone();
    right.id = id("itm_right");
    right.record_range.start = time(1200);
    set_source_start(&mut right, time(1200));
    track.clips.extend([middle, right]);
    project
}

fn batch(project: &ProjectEnvelope, operation_id: &str, operation: EditOperation) -> EditBatch {
    EditBatch {
        operation_id: OperationId::new(operation_id).unwrap(),
        base_revision: project.project.revision,
        atomic: true,
        preconditions: vec![],
        operations: vec![operation],
    }
}

fn applied(outcome: EditOutcome) -> ProjectEnvelope {
    let EditOutcome::Applied { project, .. } = outcome else {
        panic!("edit should apply")
    };
    project
}

fn id(value: &str) -> ItemId {
    ItemId::new(value).unwrap()
}

fn set_source_start(clip: &mut Clip, value: RationalTime) {
    let SourceTimeMap::Linear { source_start, .. } =
        &mut clip.source_mapping.as_mut().unwrap().time_map
    else {
        panic!("linear mapping")
    };
    *source_start = value;
}

fn source_start(clip: &Clip) -> RationalTime {
    let SourceTimeMap::Linear { source_start, .. } =
        &clip.source_mapping.as_ref().unwrap().time_map
    else {
        panic!("linear mapping")
    };
    *source_start
}

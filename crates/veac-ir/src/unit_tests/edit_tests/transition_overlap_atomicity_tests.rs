use crate::test_support::{add_transition, time};

use super::*;

#[test]
fn duration_only_edit_is_rejected_without_publishing_partial_state() {
    let project = project_with_transition();
    let before = project.clone();
    let outcome = apply_edit_batch(
        &project,
        &batch(
            "op_transition_duration_only",
            &project,
            vec![set_transition(31)],
        ),
    );
    assert_rejected(outcome, "TRANSITION_DURATION_MISMATCH");
    assert_eq!(project, before);
}

#[test]
fn endpoint_range_and_duration_can_change_in_one_atomic_batch() {
    let project = project_with_transition();
    let outcome = apply_edit_batch(
        &project,
        &batch(
            "op_transition_overlap_atomic",
            &project,
            vec![
                EditOperation::MoveClip {
                    clip_id: item("itm_middle"),
                    record_start: time(560),
                },
                set_transition(40),
            ],
        ),
    );
    let updated = applied(outcome);
    assert_eq!(
        updated.project.sequences[0].tracks[0].clips[1]
            .record_range
            .start,
        time(560)
    );
    assert_eq!(
        crate::test_support::transition_from(&updated, "itm_video")
            .unwrap()
            .duration,
        time(40)
    );
}

fn project_with_transition() -> ProjectEnvelope {
    let mut project = transition_ready_project(30);
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_middle",
        transition(30),
    );
    project
}

fn set_transition(duration: i64) -> EditOperation {
    EditOperation::SetTransition {
        clip_id: item("itm_video"),
        transition: Some(transition(duration)),
    }
}

fn transition(duration: i64) -> Transition {
    Transition {
        kind: TransitionKind::Dissolve,
        duration: time(duration),
        alignment: TransitionAlignment::Centered,
    }
}

fn item(value: &str) -> ItemId {
    ItemId::new(value).unwrap()
}

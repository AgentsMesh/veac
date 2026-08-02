use crate::test_support::add_transition;

use super::*;

#[test]
fn transition_crud_preserves_identity_and_reports_relation_changes() {
    let project = magnetic_project();
    let created = apply_edit_batch(
        &project,
        &batch("op_transition_create", &project, vec![set("itm_video", 30)]),
    );
    let (created, changes) = applied_changes(created);
    let relation_id = RelationId::new("rel_transition_itm_video").unwrap();
    assert_eq!(created.project.relations[0].id, relation_id);
    assert!(changes.contains(&ChangedObjectId::Relation {
        id: relation_id.clone()
    }));

    let updated = applied(apply_edit_batch(
        &created,
        &batch("op_transition_update", &created, vec![set("itm_video", 60)]),
    ));
    assert_eq!(updated.project.relations[0].id, relation_id);
    let transition = crate::test_support::transition_from(&updated, "itm_video").unwrap();
    assert_eq!(transition.duration, time(60));

    let removed = applied(apply_edit_batch(
        &updated,
        &batch(
            "op_transition_remove",
            &updated,
            vec![EditOperation::SetTransition {
                clip_id: id("itm_video"),
                transition: None,
            }],
        ),
    ));
    assert!(removed.project.relations.is_empty());
}

#[test]
fn locked_transition_endpoints_reject_updates() {
    let mut project = transition_project();
    project.project.sequences[0].tracks[0].state.locked = true;
    let before = project.clone();
    assert_rejected(
        apply_edit_batch(
            &project,
            &batch("op_locked_transition", &project, vec![set("itm_video", 30)]),
        ),
        "EDIT_REJECTED",
    );
    assert_eq!(project, before);
}

#[test]
fn remove_and_ripple_delete_drop_all_incident_transitions() {
    for ripple in [false, true] {
        let mut project = transition_project();
        if !ripple {
            project.project.sequences[0].tracks[0].placement_mode = PlacementMode::Free;
        }
        let operation = if ripple {
            EditOperation::RippleDelete {
                clip_id: id("itm_middle"),
            }
        } else {
            EditOperation::RemoveClip {
                clip_id: id("itm_middle"),
            }
        };
        let result = applied(apply_edit_batch(
            &project,
            &batch(&format!("op_delete_{ripple}"), &project, vec![operation]),
        ));
        assert!(result.project.relations.is_empty());
        if ripple {
            assert_eq!(
                result.project.sequences[0].tracks[0].clips[1]
                    .record_range
                    .start,
                time(600)
            );
        }
    }
}

#[test]
fn split_keeps_incoming_and_moves_outgoing_to_the_right_fragment() {
    let project = transition_project();
    let result = applied(apply_edit_batch(
        &project,
        &batch(
            "op_transition_split",
            &project,
            vec![EditOperation::SplitClip {
                clip_id: id("itm_middle"),
                at: time(900),
                right_clip_id: id("itm_middle_right"),
                relation_fragments: vec![],
            }],
        ),
    ));
    assert_edge(&result, "rel_transition_0", "itm_video", "itm_middle");
    assert_edge(&result, "rel_transition_1", "itm_middle_right", "itm_right");
    assert!(RelationGraph::project(&result.project)
        .transition_from(&SequenceId::new("seq_main").unwrap(), &id("itm_middle"))
        .is_none());
}

#[test]
fn a_later_failure_rolls_back_split_relation_rewrites() {
    let project = transition_project();
    let before = project.clone();
    let operations = vec![
        EditOperation::SplitClip {
            clip_id: id("itm_middle"),
            at: time(900),
            right_clip_id: id("itm_middle_right"),
            relation_fragments: vec![],
        },
        EditOperation::MoveClip {
            clip_id: id("itm_missing"),
            record_start: time(0),
        },
    ];
    assert_rejected(
        apply_edit_batch(
            &project,
            &batch("op_transition_rollback", &project, operations),
        ),
        "EDIT_REJECTED",
    );
    assert_eq!(project, before);
}

fn transition_project() -> ProjectEnvelope {
    let mut project = magnetic_project();
    add_transition(
        &mut project,
        "seq_main",
        "itm_video",
        "itm_middle",
        dissolve(30),
    );
    add_transition(
        &mut project,
        "seq_main",
        "itm_middle",
        "itm_right",
        dissolve(30),
    );
    project
}

fn set(clip: &str, duration: i64) -> EditOperation {
    EditOperation::SetTransition {
        clip_id: id(clip),
        transition: Some(dissolve(duration)),
    }
}

fn dissolve(duration: i64) -> Transition {
    Transition {
        kind: TransitionKind::Dissolve,
        duration: time(duration),
        alignment: TransitionAlignment::Centered,
    }
}

fn id(value: &str) -> ItemId {
    ItemId::new(value).unwrap()
}

fn assert_edge(project: &ProjectEnvelope, relation: &str, from: &str, to: &str) {
    let relation = project
        .project
        .relations
        .iter()
        .find(|value| value.id.as_str() == relation)
        .unwrap();
    let RelationKind::Transition { from: a, to: b, .. } = &relation.kind else {
        panic!("expected transition")
    };
    assert_eq!(a.item_id().unwrap().as_str(), from);
    assert_eq!(b.item_id().unwrap().as_str(), to);
}

fn applied_changes(outcome: EditOutcome) -> (ProjectEnvelope, Vec<ChangedObjectId>) {
    match outcome {
        EditOutcome::Applied {
            project,
            changed_objects,
            ..
        } => (project, changed_objects),
        other => panic!("expected applied, got {other:?}"),
    }
}

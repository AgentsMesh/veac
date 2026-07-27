use super::*;
use crate::test_support::linked_project;

#[test]
fn group_members_propagate_track_locks() {
    let mut envelope = linked_project();
    let sequence_id = envelope.project.sequences[0].id.clone();
    let item_id = envelope.project.sequences[0].tracks[0].clips[0].id.clone();
    let relation = Relation {
        id: RelationId::new("rel_lock_group").expect("relation id"),
        sequence_id: sequence_id.clone(),
        kind: RelationKind::Group {
            members: vec![RelationEndpoint::item(item_id.clone())],
        },
    };
    envelope.project.relations.push(relation);
    envelope.project.sequences[0].tracks[0].state.locked = true;
    let relation = envelope.project.relations.last().unwrap().clone();
    assert!(ensure_relation_unlocked(&envelope.project, &relation).is_err());
}

#[test]
fn missing_track_and_apply_endpoints_are_rejected() {
    let envelope = crate::test_support::sample_project();
    let sequence_id = envelope.project.sequences[0].id.clone();
    let missing_track = relation(
        "rel_missing_track",
        &sequence_id,
        RelationEndpoint::track(TrackId::new("trk_missing").unwrap()),
    );
    assert!(ensure_relation_unlocked(&envelope.project, &missing_track).is_err());

    let missing_apply = relation(
        "rel_missing_apply",
        &sequence_id,
        RelationEndpoint::apply(ApplyId::new("apl_missing").unwrap()),
    );
    assert!(ensure_relation_unlocked(&envelope.project, &missing_apply).is_err());
}

fn relation(id: &str, sequence_id: &SequenceId, endpoint: RelationEndpoint) -> Relation {
    Relation {
        id: RelationId::new(id).unwrap(),
        sequence_id: sequence_id.clone(),
        kind: RelationKind::Group {
            members: vec![endpoint],
        },
    }
}

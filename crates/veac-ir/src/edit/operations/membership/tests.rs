use super::*;
use crate::test_support::linked_project;

fn iid(value: &str) -> ItemId {
    ItemId::new(value).unwrap()
}

fn group(id: &str, members: &[&str]) -> crate::Relation {
    crate::Relation {
        id: RelationId::new(id).unwrap(),
        sequence_id: SequenceId::new("seq_main").unwrap(),
        kind: RelationKind::Group {
            members: members
                .iter()
                .map(|id| RelationEndpoint::Item { item_id: iid(id) })
                .collect(),
        },
    }
}

#[test]
fn connected_and_linked_read_canonical_relations() {
    let mut envelope = linked_project();
    envelope
        .project
        .relations
        .push(group("rel_group", &["itm_audio", "itm_caption"]));

    let (_, connected_ids) = connected(&envelope.project, &iid("itm_video")).unwrap();
    let linked_ids = linked(&envelope.project, &iid("itm_video")).unwrap();
    assert_eq!(
        connected_ids,
        vec![iid("itm_audio"), iid("itm_caption"), iid("itm_video")]
    );
    assert_eq!(linked_ids, vec![iid("itm_audio"), iid("itm_video")]);
}

#[test]
fn detach_removed_shrinks_or_discards_membership_relations() {
    let mut envelope = linked_project();
    envelope.project.relations.push(group(
        "rel_group",
        &["itm_video", "itm_audio", "itm_caption"],
    ));
    let mut changed = ChangeSet::new();
    detach_removed(
        &mut envelope.project,
        &SequenceId::new("seq_main").unwrap(),
        &BTreeSet::from([iid("itm_video")]),
        &mut changed,
    )
    .unwrap();

    assert!(envelope
        .project
        .relations
        .iter()
        .all(|relation| !matches!(relation.kind, RelationKind::AvLink { .. })));
    let group = envelope
        .project
        .relations
        .iter()
        .find(|relation| relation.id.as_str() == "rel_group")
        .unwrap();
    let RelationKind::Group { members } = &group.kind else {
        panic!("expected group relation")
    };
    assert_eq!(members.len(), 2);
    assert!(changed.contains(&crate::ChangedObjectId::Relation {
        id: RelationId::new("rel_group").unwrap(),
    }));
}

#[test]
fn detach_removed_discards_a_group_with_fewer_than_two_members() {
    let mut envelope = linked_project();
    envelope
        .project
        .relations
        .push(group("rel_group", &["itm_video", "itm_caption"]));
    let mut changed = ChangeSet::new();
    detach_removed(
        &mut envelope.project,
        &SequenceId::new("seq_main").unwrap(),
        &BTreeSet::from([iid("itm_caption")]),
        &mut changed,
    )
    .unwrap();
    assert!(envelope
        .project
        .relations
        .iter()
        .all(|relation| relation.id.as_str() != "rel_group"));
}

#[test]
fn membership_queries_reject_unknown_items() {
    let envelope = crate::test_support::sample_project();
    let missing = iid("itm_missing");
    assert!(connected(&envelope.project, &missing).is_err());
    assert!(linked(&envelope.project, &missing).is_err());
    assert!(ensure_independent(&envelope.project, &missing).is_err());
    assert!(track_id(&envelope.project, &missing).is_err());
}

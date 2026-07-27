use super::*;
use crate::test_support::sample_project;

fn group(sequence_id: SequenceId, id: &str) -> Relation {
    Relation {
        id: RelationId::new(id).expect("relation id"),
        sequence_id,
        kind: RelationKind::Group {
            members: Vec::new(),
        },
    }
}

#[test]
fn rejects_unknown_sequence_and_invalid_relation_graph() {
    let mut envelope = sample_project();
    let mut changed = ChangeSet::default();
    let unknown = group(
        SequenceId::new("seq_missing").expect("sequence id"),
        "rel_unknown_sequence",
    );
    assert!(insert(&mut envelope.project, &unknown, &mut changed).is_err());

    let invalid = group(
        envelope.project.sequences[0].id.clone(),
        "rel_invalid_group",
    );
    assert!(insert(&mut envelope.project, &invalid, &mut changed).is_err());
}

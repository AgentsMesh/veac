use super::*;
use crate::test_support::sample_project;

#[test]
fn rejects_duplicate_relation_ids_and_invalid_scoped_graphs() {
    let mut envelope = sample_project();
    let mut sequence = envelope.project.sequences[0].clone();
    sequence.id = SequenceId::new("seq_insert_test").expect("sequence id");
    let mut changed = ChangeSet::default();
    let existing = Relation {
        id: RelationId::new("rel_duplicate_insert").expect("relation id"),
        sequence_id: envelope.project.sequences[0].id.clone(),
        kind: RelationKind::Group {
            members: Vec::new(),
        },
    };
    envelope.project.relations.push(existing.clone());
    let mut duplicate = existing;
    duplicate.sequence_id = sequence.id.clone();
    assert!(insert_sequence(
        &mut envelope.project,
        &sequence,
        &[duplicate],
        &None,
        &None,
        &mut changed,
    )
    .is_err());

    let invalid = Relation {
        id: RelationId::new("rel_invalid_insert").expect("relation id"),
        sequence_id: sequence.id.clone(),
        kind: RelationKind::Group {
            members: Vec::new(),
        },
    };
    assert!(insert_sequence(
        &mut envelope.project,
        &sequence,
        &[invalid],
        &None,
        &None,
        &mut changed,
    )
    .is_err());
}

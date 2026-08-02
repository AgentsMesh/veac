use veac_plan::canonical::*;

pub fn add_transition(
    envelope: &mut ProjectEnvelope,
    sequence: &str,
    from: &str,
    to: &str,
    transition: Transition,
) {
    let index = envelope.project.relations.len();
    envelope.project.relations.push(Relation {
        id: RelationId::new(format!("rel_transition_{index}")).unwrap(),
        sequence_id: SequenceId::new(sequence).unwrap(),
        kind: RelationKind::Transition {
            from: RelationEndpoint::item(ItemId::new(from).unwrap()),
            to: RelationEndpoint::item(ItemId::new(to).unwrap()),
            transition,
        },
    });
}

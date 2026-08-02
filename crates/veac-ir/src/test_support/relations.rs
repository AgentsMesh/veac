use crate::*;

pub(crate) fn add_transition(
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

pub(crate) fn transition_mut<'a>(
    envelope: &'a mut ProjectEnvelope,
    from: &str,
) -> &'a mut Transition {
    envelope
        .project
        .relations
        .iter_mut()
        .find_map(|relation| match &mut relation.kind {
            RelationKind::Transition {
                from: endpoint,
                transition,
                ..
            } if endpoint.item_id().is_some_and(|id| id.as_str() == from) => Some(transition),
            _ => None,
        })
        .expect("transition relation")
}

pub(crate) fn transition_from<'a>(
    envelope: &'a ProjectEnvelope,
    from: &str,
) -> Option<&'a Transition> {
    envelope
        .project
        .relations
        .iter()
        .find_map(|relation| match &relation.kind {
            RelationKind::Transition {
                from: endpoint,
                transition,
                ..
            } if endpoint.item_id().is_some_and(|id| id.as_str() == from) => Some(transition),
            _ => None,
        })
}

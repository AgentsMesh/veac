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

pub fn add_matte(
    envelope: &mut ProjectEnvelope,
    sequence: &str,
    producer: &str,
    consumer: &str,
    mode: TrackMatteMode,
    invert: bool,
) {
    envelope.project.relations.push(Relation {
        id: RelationId::new(format!("rel_matte_{consumer}")).unwrap(),
        sequence_id: SequenceId::new(sequence).unwrap(),
        kind: RelationKind::Matte {
            producer: RelationEndpoint::item(ItemId::new(producer).unwrap()),
            consumer: RelationEndpoint::item(ItemId::new(consumer).unwrap()),
            parameters: MatteRelationParameters { mode, invert },
        },
    });
}

pub fn add_sidechain(
    envelope: &mut ProjectEnvelope,
    sequence: &str,
    key: RelationEndpoint,
    target: &str,
    parameters: SidechainRelationParameters,
) {
    envelope.project.relations.push(Relation {
        id: RelationId::new(format!("rel_sidechain_{target}")).unwrap(),
        sequence_id: SequenceId::new(sequence).unwrap(),
        kind: RelationKind::Sidechain {
            key,
            target: RelationEndpoint::item(ItemId::new(target).unwrap()),
            parameters,
        },
    });
}

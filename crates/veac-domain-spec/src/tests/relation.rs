use super::*;

#[test]
fn centered_transition_contract_separates_topology_from_mechanism() {
    let registry = DomainOperationRegistry::standard();
    let dissolve = registry
        .lookup(DomainOperationId::TransitionDissolve)
        .unwrap();
    assert_eq!(dissolve.result(), domain(DomainType::Transition));
    assert_eq!(
        dissolve.instruction(),
        DomainInstructionKind::DomainConstruct
    );
    assert_eq!(dissolve.operands()[0].axis(), OperandAxis::Leaf);

    let relation = registry
        .lookup(DomainOperationId::RelationTransition)
        .unwrap();
    assert_eq!(relation.result(), domain(DomainType::Relation));
    assert_eq!(
        relation.runtime_action(),
        DomainRuntimeAction::RelationConstructor
    );
    assert_eq!(
        relation
            .operands()
            .iter()
            .map(DomainOperandContract::axis)
            .collect::<Vec<_>>(),
        [
            OperandAxis::Topology,
            OperandAxis::Topology,
            OperandAxis::Topology,
            OperandAxis::Leaf,
        ]
    );
    assert_eq!(
        relation.operands()[3].shape(),
        domain(DomainType::Transition)
    );
}

#[test]
fn all_relation_variants_attach_through_one_sequence_method() {
    let registry = DomainOperationRegistry::standard();
    let attach = registry
        .lookup(DomainOperationId::SequenceWithRelation)
        .unwrap();
    assert_eq!(attach.operands()[1].shape(), domain(DomainType::Relation));
    assert_eq!(
        attach.runtime_action(),
        DomainRuntimeAction::OwnedAttachment
    );
    assert_eq!(
        registry
            .lookup_method(DomainType::Sequence, "with_relation")
            .map(DomainOperationContract::id),
        Some(DomainOperationId::SequenceWithRelation)
    );
    assert!(registry
        .lookup_method(DomainType::Layer, "with_relation")
        .is_none());
}

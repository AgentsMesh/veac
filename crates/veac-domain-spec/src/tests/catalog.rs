use std::collections::BTreeSet;

use super::*;

#[test]
fn catalog_covers_the_complete_v8_algebra() {
    let registry = DomainOperationRegistry::standard();
    assert_eq!(registry.version(), DomainOpsetVersion::V8);
    assert_eq!(registry.len(), 582);
    assert_eq!(registry.len(), DomainOperationId::all().len());
    assert_eq!(DomainType::all().len(), 214);
    assert!(!registry.is_empty());

    let mut opcodes = BTreeSet::new();
    let mut names = BTreeSet::new();
    for contract in registry.contracts() {
        assert!(opcodes.insert(contract.id().opcode()));
        assert!(names.insert(contract.name()));
        assert_eq!(registry.lookup(contract.id()), Some(contract));
        assert_eq!(
            registry.lookup_opcode(contract.id().opcode()),
            Some(contract)
        );
        assert_eq!(registry.lookup_name(contract.name()), Some(contract));
        let expected = match contract.instruction() {
            DomainInstructionKind::DomainConstruct => Effect::Pure,
            DomainInstructionKind::GraphEmit => Effect::GraphEmit,
        };
        assert_eq!(contract.effect(), expected, "{}", contract.name());
    }
}

#[test]
fn classifications_keep_graph_topology_and_leaf_values_separate() {
    for value in [
        DomainType::Project,
        DomainType::Sequence,
        DomainType::Layer,
        DomainType::Item,
    ] {
        assert!(value.is_container());
        assert!(value.is_graph_entity());
        assert!(value.requires_topology_axis());
    }
    for value in [
        DomainType::Resource,
        DomainType::Relation,
        DomainType::Apply,
        DomainType::Annotation,
        DomainType::Delivery,
    ] {
        assert!(!value.is_container());
        assert!(value.is_graph_entity());
        assert!(value.requires_topology_axis());
    }
    assert!(!DomainType::Transform2D.is_graph_entity());
    assert!(!DomainType::Transform2D.requires_topology_axis());
    assert!(DomainType::Source.requires_topology_axis());
}

#[test]
fn representative_contracts_pin_axes_lists_and_graph_actions() {
    let registry = DomainOperationRegistry::standard();
    let project = registry.lookup(DomainOperationId::Project).unwrap();
    assert_eq!(project.instruction(), DomainInstructionKind::GraphEmit);
    assert!(project
        .operands()
        .iter()
        .all(|operand| operand.axis() == OperandAxis::Topology));

    let visual = registry.lookup(DomainOperationId::ItemWithVisual).unwrap();
    assert_eq!(
        visual.runtime_action(),
        DomainRuntimeAction::NonOwningUpdate
    );
    assert_eq!(visual.operands()[0].axis(), OperandAxis::Topology);
    assert_eq!(visual.operands()[1].axis(), OperandAxis::Leaf);

    let matrix = registry.lookup(DomainOperationId::RgbMatrix).unwrap();
    assert_eq!(
        matrix.operands()[0].shape(),
        DomainValueShape::primitive_list(PrimitiveType::Scalar)
    );
    let curve = registry.lookup(DomainOperationId::SourceTimeCurve).unwrap();
    assert_eq!(
        curve.operands()[0].shape(),
        DomainValueShape::domain_list(DomainType::SourceTimeSegment)
    );
}

#[test]
fn every_owner_boundary_has_a_pinned_atomic_plural_contract() {
    let registry = DomainOperationRegistry::standard();
    let pairs = [
        (
            DomainOperationId::ProjectWithResource,
            DomainOperationId::ProjectWithResources,
            DomainType::Project,
            DomainType::Resource,
            0x2025,
        ),
        (
            DomainOperationId::ProjectWithSequence,
            DomainOperationId::ProjectWithSequences,
            DomainType::Project,
            DomainType::Sequence,
            0x2026,
        ),
        (
            DomainOperationId::ProjectWithMulticamGroup,
            DomainOperationId::ProjectWithMulticamGroups,
            DomainType::Project,
            DomainType::MulticamGroup,
            0x9008,
        ),
        (
            DomainOperationId::ProjectWithAnnotation,
            DomainOperationId::ProjectWithAnnotations,
            DomainType::Project,
            DomainType::Annotation,
            0x9034,
        ),
        (
            DomainOperationId::ProjectWithDelivery,
            DomainOperationId::ProjectWithDeliveries,
            DomainType::Project,
            DomainType::Delivery,
            0xa051,
        ),
        (
            DomainOperationId::SequenceWithLayer,
            DomainOperationId::SequenceWithLayers,
            DomainType::Sequence,
            DomainType::Layer,
            0x2028,
        ),
        (
            DomainOperationId::SequenceWithRelation,
            DomainOperationId::SequenceWithRelations,
            DomainType::Sequence,
            DomainType::Relation,
            0x801f,
        ),
        (
            DomainOperationId::SequenceWithApply,
            DomainOperationId::SequenceWithApplies,
            DomainType::Sequence,
            DomainType::Apply,
            0x802d,
        ),
        (
            DomainOperationId::LayerWithItem,
            DomainOperationId::LayerWithItems,
            DomainType::Layer,
            DomainType::Item,
            0x2029,
        ),
    ];
    for (index, (single, plural, owner, child, old_opcode)) in pairs.into_iter().enumerate() {
        assert_eq!(single.opcode(), old_opcode);
        assert_eq!(plural.opcode(), 0x202a + index as u16);
        assert_owner_contract(registry.lookup(single).unwrap(), owner, child, false);
        assert_owner_contract(registry.lookup(plural).unwrap(), owner, child, true);
    }
}

fn assert_owner_contract(
    contract: &DomainOperationContract,
    owner: DomainType,
    child: DomainType,
    plural: bool,
) {
    assert_eq!(contract.exposure().receiver(), Some(owner));
    assert_eq!(
        contract.operands()[0].shape(),
        DomainValueShape::domain(owner)
    );
    let expected = if plural {
        DomainValueShape::domain_list(child)
    } else {
        DomainValueShape::domain(child)
    };
    assert_eq!(contract.operands()[1].shape(), expected);
    assert!(contract
        .operands()
        .iter()
        .all(|value| value.axis() == OperandAxis::Topology));
    assert_eq!(contract.result(), DomainValueShape::domain(owner));
    assert_eq!(contract.instruction(), DomainInstructionKind::GraphEmit);
    assert_eq!(
        contract.runtime_action(),
        DomainRuntimeAction::OwnedAttachment
    );
    assert_eq!(contract.effect(), Effect::GraphEmit);
    assert_eq!(contract.max_stage(), Stage::Build);
}

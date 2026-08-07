use std::collections::BTreeSet;

use crate::program::domain_system::{
    DomainInstructionKind, DomainOperationExposure, DomainOperationId, DomainOperationRegistry,
    DomainRuntimeAction, DomainType, DomainValueShape, OperandAxis,
};
use crate::program::expression::{Effect, Stage};

type Expected = (DomainType, &'static str, DomainType, DomainOperationId, u16);

#[test]
fn plural_owned_method_contracts_are_an_exact_closed_surface() {
    let registry = DomainOperationRegistry::standard();
    let expected = expected_contracts();
    let actual = registry
        .contracts()
        .filter(|contract| {
            contract.runtime_action() == DomainRuntimeAction::OwnedAttachment
                && matches!(
                    contract.operands().get(1).map(|operand| operand.shape()),
                    Some(DomainValueShape::DomainList(_))
                )
        })
        .map(|contract| {
            let DomainOperationExposure::Method { receiver, name } = contract.exposure() else {
                panic!(
                    "plural owned attachment is not a method: {}",
                    contract.name()
                );
            };
            let DomainValueShape::DomainList(child) = contract.operands()[1].shape() else {
                unreachable!();
            };
            (
                *receiver,
                name.to_string(),
                child,
                contract.id(),
                contract.id().opcode(),
            )
        })
        .collect::<BTreeSet<_>>();
    let expected_surface = expected
        .iter()
        .map(|(receiver, name, child, id, opcode)| {
            (*receiver, (*name).to_owned(), *child, *id, *opcode)
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(actual, expected_surface);

    for (receiver, method, child, id, opcode) in expected {
        assert_plural_contract(&registry, receiver, method, child, id, opcode);
    }
}

fn assert_plural_contract(
    registry: &DomainOperationRegistry,
    receiver: DomainType,
    method: &str,
    child: DomainType,
    id: DomainOperationId,
    opcode: u16,
) {
    let contract = registry
        .lookup_method(receiver, method)
        .unwrap_or_else(|| panic!("missing {receiver}.{method}"));
    assert_eq!(contract.id(), id);
    assert_eq!(contract.id().opcode(), opcode);
    assert_eq!(contract.exposure().receiver(), Some(receiver));
    assert_eq!(contract.exposure().name(), method);
    assert_eq!(contract.operands().len(), 2);
    assert_eq!(
        contract.operands()[0].shape(),
        DomainValueShape::Domain(receiver)
    );
    assert_eq!(
        contract.operands()[1].shape(),
        DomainValueShape::DomainList(child)
    );
    assert!(contract
        .operands()
        .iter()
        .all(|operand| operand.axis() == OperandAxis::Topology));
    assert_eq!(contract.result(), DomainValueShape::Domain(receiver));
    assert_eq!(contract.instruction(), DomainInstructionKind::GraphEmit);
    assert_eq!(contract.effect(), Effect::GraphEmit);
    assert_eq!(contract.max_stage(), Stage::Build);
    assert_eq!(
        contract.runtime_action(),
        DomainRuntimeAction::OwnedAttachment
    );
}

fn expected_contracts() -> [Expected; 9] {
    use DomainOperationId as Op;
    use DomainType as Type;
    [
        (
            Type::Project,
            "with_resources",
            Type::Resource,
            Op::ProjectWithResources,
            0x202a,
        ),
        (
            Type::Project,
            "with_sequences",
            Type::Sequence,
            Op::ProjectWithSequences,
            0x202b,
        ),
        (
            Type::Project,
            "with_multicam_groups",
            Type::MulticamGroup,
            Op::ProjectWithMulticamGroups,
            0x202c,
        ),
        (
            Type::Project,
            "with_annotations",
            Type::Annotation,
            Op::ProjectWithAnnotations,
            0x202d,
        ),
        (
            Type::Project,
            "with_deliveries",
            Type::Delivery,
            Op::ProjectWithDeliveries,
            0x202e,
        ),
        (
            Type::Sequence,
            "with_layers",
            Type::Layer,
            Op::SequenceWithLayers,
            0x202f,
        ),
        (
            Type::Sequence,
            "with_relations",
            Type::Relation,
            Op::SequenceWithRelations,
            0x2030,
        ),
        (
            Type::Sequence,
            "with_applies",
            Type::Apply,
            Op::SequenceWithApplies,
            0x2031,
        ),
        (
            Type::Layer,
            "with_items",
            Type::Item,
            Op::LayerWithItems,
            0x2032,
        ),
    ]
}

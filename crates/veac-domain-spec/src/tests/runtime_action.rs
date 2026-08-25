use std::collections::BTreeMap;

use super::*;

#[test]
fn every_operation_declares_one_closed_runtime_action() {
    let mut counts = BTreeMap::new();
    for contract in all_contracts() {
        *counts.entry(contract.runtime_action()).or_insert(0usize) += 1;
    }
    assert_eq!(counts[&DomainRuntimeAction::Description], 535);
    assert_eq!(counts[&DomainRuntimeAction::EntityConstructor], 17);
    assert_eq!(counts[&DomainRuntimeAction::OwnedAttachment], 18);
    assert_eq!(counts[&DomainRuntimeAction::NonOwningUpdate], 4);
    assert_eq!(counts[&DomainRuntimeAction::ProjectEntry], 1);
    assert_eq!(counts[&DomainRuntimeAction::RelationConstructor], 7);
    assert_eq!(
        counts.values().sum::<usize>(),
        DomainOperationId::all().len()
    );
    assert_eq!(counts.len(), 6);
}

#[test]
fn runtime_action_must_match_instruction_and_effect() {
    let value = builtin(DomainOperationId::Canvas);
    let corrupt = operation_with_exposure(
        value.id(),
        value.name(),
        value.exposure().clone(),
        value.operands().to_vec(),
        value.result(),
        DomainOperationSemantics::new(
            value.instruction(),
            DomainRuntimeAction::EntityConstructor,
            value.effect(),
            value.max_stage(),
            value.temporal_lowering(),
        ),
    );
    let error = rebuild([corrupt], MAX_DOMAIN_OPERATIONS, MAX_DOMAIN_REGISTRY_BYTES).unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_EFFECT");
}

#[test]
fn explicit_runtime_semantics_are_part_of_registry_digest() {
    let original = builtin(DomainOperationId::Project);
    let changed = operation_with_exposure(
        original.id(),
        original.name(),
        original.exposure().clone(),
        original.operands().to_vec(),
        original.result(),
        DomainOperationSemantics::new(
            original.instruction(),
            DomainRuntimeAction::NonOwningUpdate,
            original.effect(),
            original.max_stage(),
            original.temporal_lowering(),
        ),
    );
    let digest = |value: &DomainOperationContract| {
        super::super::digest::registry(DomainOpsetVersion::CURRENT, std::iter::once(value))
    };

    assert_ne!(digest(&original), digest(&changed));
}

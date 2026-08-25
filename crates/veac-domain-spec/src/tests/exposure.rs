use std::collections::BTreeSet;

use super::*;

#[test]
fn every_exposure_is_unique_and_resolves_to_its_contract() {
    let registry = DomainOperationRegistry::standard();
    let mut exposures = BTreeSet::new();
    let mut functions = 0;
    let mut methods = 0;
    for contract in registry.contracts() {
        assert!(exposures.insert(contract.exposure().clone()));
        let resolved = match contract.exposure() {
            DomainOperationExposure::FreeFunction { name } => {
                functions += 1;
                registry.lookup_function(name)
            }
            DomainOperationExposure::Method { receiver, name } => {
                methods += 1;
                registry.lookup_method(*receiver, name)
            }
        };
        assert_eq!(resolved, Some(contract));
    }
    assert_eq!((functions, methods, exposures.len()), (559, 23, 582));
}

#[test]
fn method_lookup_uses_the_declared_receiver_identity() {
    let registry = DomainOperationRegistry::standard();
    for contract in registry.contracts() {
        let DomainOperationExposure::Method { receiver, name } = contract.exposure() else {
            continue;
        };
        assert_eq!(
            contract.operands().first().map(|operand| operand.shape()),
            Some(DomainValueShape::Domain(*receiver))
        );
        for other in DomainType::all().filter(|value| value != receiver) {
            assert!(registry.lookup_method(other, name).is_none());
        }
    }
}

#[test]
fn non_exposed_and_nonexistent_calls_fail_closed() {
    let registry = DomainOperationRegistry::standard();
    assert!(registry.lookup_function("project_with_sequence").is_none());
    assert!(registry.lookup_function("with_sequence").is_none());
    assert!(registry
        .lookup_method(DomainType::Project, "project_with_sequence")
        .is_none());
    assert!(registry
        .lookup_method(DomainType::Sequence, "with_sequence")
        .is_none());
    assert!(registry.lookup_function("nonexistent").is_none());
    assert!(registry
        .lookup_method(DomainType::Project, "nonexistent")
        .is_none());
}

#[test]
fn duplicate_function_and_method_exposures_are_rejected() {
    let duplicate_function = reexposed(
        DomainOperationId::FrameRate,
        DomainOperationExposure::free_function("canvas"),
    );
    let error = rebuild(
        [builtin(DomainOperationId::Canvas), duplicate_function],
        MAX_DOMAIN_OPERATIONS,
        MAX_DOMAIN_REGISTRY_BYTES,
    )
    .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_DUPLICATE_EXPOSURE");

    let duplicate_method = reexposed(
        DomainOperationId::ProjectWithResource,
        DomainOperationExposure::method(DomainType::Project, "with_sequence"),
    );
    let error = rebuild(
        [
            builtin(DomainOperationId::ProjectWithSequence),
            duplicate_method,
        ],
        MAX_DOMAIN_OPERATIONS,
        MAX_DOMAIN_REGISTRY_BYTES,
    )
    .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_DUPLICATE_EXPOSURE");
}

#[test]
fn receiver_drift_is_rejected_and_surface_changes_the_digest() {
    let receiver_drift = reexposed(
        DomainOperationId::ProjectWithSequence,
        DomainOperationExposure::method(DomainType::Sequence, "with_sequence"),
    );
    let error = rebuild(
        [receiver_drift],
        MAX_DOMAIN_OPERATIONS,
        MAX_DOMAIN_REGISTRY_BYTES,
    )
    .unwrap_err();
    assert_eq!(error.code(), "DOMAIN_OPERATION_EXPOSURE");

    let base = builtin(DomainOperationId::Canvas);
    let renamed = reexposed(
        DomainOperationId::Canvas,
        DomainOperationExposure::free_function("canvas_changed"),
    );
    let method = reexposed(
        DomainOperationId::Canvas,
        DomainOperationExposure::method(DomainType::Canvas, "canvas"),
    );
    let other_receiver = reexposed(
        DomainOperationId::Canvas,
        DomainOperationExposure::method(DomainType::Project, "canvas"),
    );
    let digests = [base, renamed, method, other_receiver]
        .iter()
        .map(|value| {
            super::super::digest::registry(DomainOpsetVersion::CURRENT, std::iter::once(value))
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(digests.len(), 4);
}

fn reexposed(id: DomainOperationId, exposure: DomainOperationExposure) -> DomainOperationContract {
    let value = builtin(id);
    operation_with_exposure(
        id,
        value.name(),
        exposure,
        value.operands().to_vec(),
        value.result(),
        DomainOperationSemantics::new(
            value.instruction(),
            value.runtime_action(),
            value.effect(),
            value.max_stage(),
            value.temporal_lowering(),
        ),
    )
}

use super::super::resource::{Resource, RESOURCES};
use super::super::{
    MAX_COLLECTION_BYTES, MAX_COLLECTION_ELEMENTS, MAX_EMITTED_BYTES, MAX_EMITTED_ENTITIES,
    MAX_EVALUATED_VALUE_BYTES, MAX_EVALUATOR_STORAGE_BYTES, MAX_EXECUTION_FUEL,
    MAX_EXECUTION_ITERATIONS, MAX_RESIDUAL_BYTES, MAX_RESIDUAL_NODES,
};
use super::{budget, delta, ResourceLimits};

#[test]
fn defaults_cover_every_documented_resource_dimension() {
    let limits = ResourceLimits::default();
    assert_eq!(limits.fuel, MAX_EXECUTION_FUEL);
    assert_eq!(limits.value_bytes, MAX_EVALUATED_VALUE_BYTES);
    assert_eq!(limits.iterations, MAX_EXECUTION_ITERATIONS);
    assert_eq!(limits.collection_elements, MAX_COLLECTION_ELEMENTS);
    assert_eq!(limits.collection_bytes, MAX_COLLECTION_BYTES);
    assert_eq!(limits.emitted_entities, MAX_EMITTED_ENTITIES);
    assert_eq!(limits.emitted_bytes, MAX_EMITTED_BYTES);
    assert_eq!(limits.residual_nodes, MAX_RESIDUAL_NODES);
    assert_eq!(limits.residual_bytes, MAX_RESIDUAL_BYTES);
    assert_eq!(limits.evaluator_storage_bytes, MAX_EVALUATOR_STORAGE_BYTES);
}

#[test]
fn every_resource_accepts_its_exact_limit_and_rejects_the_next_unit() {
    for resource in RESOURCES {
        let budget = budget(resource, 2);
        budget.reserve(delta(resource, 2), 4..8).unwrap();
        let error = budget.reserve(delta(resource, 1), 9..12).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
        assert!(error.message().contains(resource.label()));
        assert!(error.message().contains("2"));
        assert_eq!(error.span(), 9..12);
    }
}

#[test]
fn fresh_compilations_receive_independent_resource_ledgers() {
    let first = budget(Resource::EmittedEntities, 1);
    first
        .reserve(delta(Resource::EmittedEntities, 1), 0..1)
        .unwrap();
    first
        .reserve(delta(Resource::EmittedEntities, 1), 2..3)
        .unwrap_err();

    let second = budget(Resource::EmittedEntities, 1);
    second
        .reserve(delta(Resource::EmittedEntities, 1), 4..5)
        .unwrap();
}

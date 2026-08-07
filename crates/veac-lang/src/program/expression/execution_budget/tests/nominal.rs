use super::super::resource::Resource;
use super::super::{
    ExecutionBudget, LOGICAL_NOMINAL_BASE_BYTES, LOGICAL_NOMINAL_FIELD_HANDLE_BYTES,
};
use super::{delta, unlimited};

#[test]
fn nominal_reservation_accounts_for_one_container_and_every_field_handle() {
    let mut limits = unlimited();
    limits.collection_elements = 3;
    limits.collection_bytes = LOGICAL_NOMINAL_BASE_BYTES + 3 * LOGICAL_NOMINAL_FIELD_HANDLE_BYTES;
    ExecutionBudget::with_resource_limits(limits)
        .reserve_nominal(3, 1..2)
        .unwrap();

    let mut empty = unlimited();
    empty.collection_bytes = LOGICAL_NOMINAL_BASE_BYTES;
    ExecutionBudget::with_resource_limits(empty)
        .reserve_nominal(0, 2..3)
        .unwrap();
}

#[test]
fn nominal_failure_is_atomic_across_bytes_and_elements() {
    let mut limits = unlimited();
    limits.collection_elements = 2;
    limits.collection_bytes = LOGICAL_NOMINAL_BASE_BYTES;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let error = budget.reserve_nominal(2, 3..4).unwrap_err();
    assert!(error.message().contains("collection byte"));
    budget
        .reserve(delta(Resource::CollectionElements, 2), 4..5)
        .unwrap();
}

#[test]
fn nominal_size_overflow_does_not_commit_elements() {
    let mut limits = unlimited();
    limits.collection_elements = 1;
    let budget = ExecutionBudget::with_resource_limits(limits);
    budget.reserve_nominal(usize::MAX, 5..6).unwrap_err();
    budget
        .reserve(delta(Resource::CollectionElements, 1), 6..7)
        .unwrap();
}

#[test]
fn nominal_admission_reserves_the_complete_nested_value_atomically() {
    let (registry, id) = registry();
    let value = crate::program::expression::Value::structure(
        &registry,
        id,
        vec![crate::program::expression::Value::Text("payload".into())],
    )
    .unwrap();
    let mut exact = unlimited();
    exact.value_bytes = 7;
    exact.collection_elements = 1;
    exact.collection_bytes = LOGICAL_NOMINAL_BASE_BYTES + LOGICAL_NOMINAL_FIELD_HANDLE_BYTES;
    ExecutionBudget::with_resource_limits(exact)
        .admit_value(&value, 7..8)
        .unwrap();

    let mut short = unlimited();
    short.value_bytes = 7;
    short.collection_elements = 1;
    short.collection_bytes = exact.collection_bytes - 1;
    let budget = ExecutionBudget::with_resource_limits(short);
    budget.admit_value(&value, 8..9).unwrap_err();
    budget
        .reserve(delta(Resource::ValueBytes, 7), 9..10)
        .unwrap();
    budget
        .reserve(delta(Resource::CollectionElements, 1), 10..11)
        .unwrap();
}

fn registry() -> (crate::program::TypeRegistry, crate::program::TypeId) {
    use std::sync::Arc;

    use crate::program::expression::{PrimitiveType, ValueType};
    use crate::program::{
        FieldDefinition, FieldIndex, StructDefinition, TypeDefinition, TypeDefinitionKind,
        TypeRegistryBuilder,
    };

    let definition = TypeDefinition::new(
        "budget.veac",
        "Payload",
        TypeDefinitionKind::Struct(StructDefinition::new(vec![FieldDefinition::new(
            FieldIndex::new(0),
            "text",
            ValueType::primitive(PrimitiveType::Text),
        )])),
    );
    let id = definition.type_ref().id();
    let mut builder = TypeRegistryBuilder::new();
    builder.insert(Arc::new(definition)).unwrap();
    (builder.finish().unwrap(), id)
}

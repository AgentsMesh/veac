use std::sync::Arc;

use super::super::resource::Resource;
use super::super::{
    ExecutionBudget, LOGICAL_COLLECTION_BASE_BYTES, LOGICAL_COLLECTION_HANDLE_BYTES,
};
use super::{budget, delta, unlimited};
use crate::program::expression::{
    evaluate_lookup_with_budget, Environment, ExpressionContext, MapKeyType, PrimitiveType, Value,
    MAX_TEXT_VALUE_BYTES,
};

const ELEMENTS: usize = 4;
const CONTAINER_BYTES: usize =
    3 * LOGICAL_COLLECTION_BASE_BYTES + 5 * LOGICAL_COLLECTION_HANDLE_BYTES;
const LEAF_BYTES: usize = 6;

fn structural_value() -> Value {
    let list = Value::list(
        PrimitiveType::Text.into(),
        vec![Value::Text(Arc::from("abc"))],
    )
    .unwrap();
    let map = Value::map(
        MapKeyType::Text,
        PrimitiveType::Identifier.into(),
        vec![(Value::Text(Arc::from("k")), Value::Identifier("id".into()))],
    )
    .unwrap();
    Value::tuple(vec![list, map]).unwrap()
}

fn evaluate_input(value: Value, budget: &ExecutionBudget) -> Result<Value, String> {
    let mut environment = Environment::new();
    environment.insert("input".to_owned(), value);
    evaluate_lookup_with_budget("input", &environment, &ExpressionContext::empty(), budget)
        .map_err(|error| format!("{}:{}", error.code(), error.message()))
}

#[test]
fn empty_collections_pay_a_fixed_container_base() {
    for reserve in [
        ExecutionBudget::reserve_sequence_collection,
        ExecutionBudget::reserve_map_collection,
    ] {
        let exact = budget(Resource::CollectionBytes, LOGICAL_COLLECTION_BASE_BYTES);
        reserve(&exact, 0, 1..2).unwrap();
        let error = reserve(&exact, 0, 3..4).unwrap_err();
        assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
        assert_eq!(error.span(), 3..4);

        let short = budget(Resource::CollectionBytes, LOGICAL_COLLECTION_BASE_BYTES - 1);
        let error = reserve(&short, 0, 5..6).unwrap_err();
        assert!(error.message().contains("collection byte"));
    }
}

#[test]
fn external_structural_admission_has_exact_recursive_boundaries() {
    let mut limits = unlimited();
    limits.collection_elements = ELEMENTS;
    limits.collection_bytes = CONTAINER_BYTES;
    limits.value_bytes = LEAF_BYTES;
    let exact = ExecutionBudget::with_resource_limits(limits);
    assert_eq!(
        evaluate_input(structural_value(), &exact).unwrap(),
        structural_value()
    );

    for (resource, limit, label) in [
        (
            Resource::CollectionElements,
            ELEMENTS - 1,
            "collection element",
        ),
        (
            Resource::CollectionBytes,
            CONTAINER_BYTES - 1,
            "collection byte",
        ),
        (Resource::ValueBytes, LEAF_BYTES - 1, "evaluated value byte"),
    ] {
        let error = evaluate_input(structural_value(), &budget(resource, limit)).unwrap_err();
        assert!(error.starts_with("EXPRESSION_EXECUTION_LIMIT:"));
        assert!(error.contains(label));
    }
}

#[test]
fn failed_external_admission_commits_no_resource_dimension() {
    let mut limits = unlimited();
    limits.collection_elements = ELEMENTS;
    limits.collection_bytes = CONTAINER_BYTES;
    limits.value_bytes = LEAF_BYTES - 1;
    let budget = ExecutionBudget::with_resource_limits(limits);
    assert!(evaluate_input(structural_value(), &budget).is_err());

    budget
        .reserve(delta(Resource::CollectionElements, ELEMENTS), 1..2)
        .unwrap();
    budget
        .reserve(delta(Resource::CollectionBytes, CONTAINER_BYTES), 2..3)
        .unwrap();
    budget
        .reserve(delta(Resource::ValueBytes, LEAF_BYTES - 1), 3..4)
        .unwrap();
}

#[test]
fn nested_oversized_text_is_rejected_before_admission() {
    let text = Value::Text("x".repeat(MAX_TEXT_VALUE_BYTES + 1).into());
    let value = Value::list(PrimitiveType::Text.into(), vec![text]).unwrap();
    let budget = ExecutionBudget::with_resource_limits(unlimited());
    let error = evaluate_input(value, &budget).unwrap_err();
    assert!(error.starts_with("EXPRESSION_TEXT_LIMIT:"));

    budget
        .reserve(delta(Resource::CollectionElements, 1), 1..2)
        .unwrap();
    budget
        .reserve(
            delta(
                Resource::CollectionBytes,
                LOGICAL_COLLECTION_BASE_BYTES + LOGICAL_COLLECTION_HANDLE_BYTES,
            ),
            2..3,
        )
        .unwrap();
}

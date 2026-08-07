use super::super::{ExecutionBudget, LOGICAL_COLLECTION_BASE_BYTES};
use super::unlimited;
use crate::program::expression::{evaluate_lookup_with_budget, Environment, ExpressionContext};

fn evaluate(source: &str, budget: &ExecutionBudget) -> crate::program::expression::ExpressionError {
    evaluate_lookup_with_budget(
        source,
        &Environment::new(),
        &ExpressionContext::empty(),
        budget,
    )
    .unwrap_err()
}

#[test]
fn reservation_failure_happens_before_the_first_callback() {
    let mut limits = unlimited();
    limits.collection_bytes = LOGICAL_COLLECTION_BASE_BYTES;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let error = evaluate(
        "map(0 .. 2, fn(value: int) -> int effect pure { 9223372036854775807 + 1 })",
        &budget,
    );
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("collection byte"));
}

#[test]
fn filter_reserves_its_input_cardinality_even_when_every_value_is_removed() {
    let mut limits = unlimited();
    limits.collection_elements = 2;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let error = evaluate(
        "filter(0 .. 3, fn(value: int) -> bool effect pure { false })",
        &budget,
    );
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("aggregate collection element"));
}

#[test]
fn nested_aggregates_share_one_iteration_ledger() {
    let mut limits = unlimited();
    limits.iterations = 5;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let source = "map(0 .. 2, fn(value: int) -> list<int> effect pure { \
        map(0 .. 2, fn(inner: int) -> int effect pure { inner }) })";
    let error = evaluate(source, &budget);
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("aggregate iteration"));
}

#[test]
fn map_iteration_reserves_temporary_tuple_shape_before_callbacks() {
    let mut limits = unlimited();
    limits.collection_elements = 5;
    let budget = ExecutionBudget::with_resource_limits(limits);
    let source = "map(#{\"b\": 2, \"a\": 1}, \
        fn(entry: (text, int)) -> int effect pure { 9223372036854775807 + 1 })";
    let error = evaluate(source, &budget);
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("aggregate collection element"));
}

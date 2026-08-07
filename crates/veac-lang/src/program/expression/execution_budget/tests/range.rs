use super::super::resource::Resource;
use super::super::{ExecutionBudget, LOGICAL_RANGE_VALUE_BYTES};
use super::{budget, unlimited};
use crate::program::expression::{
    evaluate, evaluate_lookup_with_budget, Environment, ExpressionContext, Value,
};

fn evaluate_with(source: &str, budget: &ExecutionBudget) -> Result<Value, String> {
    evaluate_lookup_with_budget(
        source,
        &Environment::new(),
        &ExpressionContext::empty(),
        budget,
    )
    .map_err(|error| format!("{}:{}", error.code(), error.message()))
}

#[test]
fn range_construction_charges_fixed_value_bytes_without_collection_or_iteration() {
    let mut limits = unlimited();
    limits.value_bytes = LOGICAL_RANGE_VALUE_BYTES;
    limits.collection_elements = 0;
    limits.collection_bytes = 0;
    limits.iterations = 0;
    let exact = ExecutionBudget::with_resource_limits(limits);
    evaluate_with("-9223372036854775808 .. 9223372036854775807", &exact).unwrap();

    let short = budget(Resource::ValueBytes, LOGICAL_RANGE_VALUE_BYTES - 1);
    let error = evaluate_with("0 .. 10", &short).unwrap_err();
    assert!(error.starts_with("EXPRESSION_EXECUTION_LIMIT:"));
    assert!(error.contains("evaluated value byte"));
}

#[test]
fn external_range_admission_is_once_per_input_and_has_the_same_cost() {
    let range = evaluate("0 .. 10 by 2", &Environment::new()).unwrap();
    assert_eq!(range.evaluated_bytes(), LOGICAL_RANGE_VALUE_BYTES);
    let mut environment = Environment::new();
    environment.insert("input".to_owned(), range);

    let mut limits = unlimited();
    limits.value_bytes = LOGICAL_RANGE_VALUE_BYTES;
    limits.collection_elements = 0;
    limits.collection_bytes = 0;
    let exact = ExecutionBudget::with_resource_limits(limits);
    let result = evaluate_lookup_with_budget(
        "input == input",
        &environment,
        &ExpressionContext::empty(),
        &exact,
    )
    .unwrap();
    assert_eq!(result, Value::Bool(true));

    let short = budget(Resource::ValueBytes, LOGICAL_RANGE_VALUE_BYTES - 1);
    let error =
        evaluate_lookup_with_budget("input", &environment, &ExpressionContext::empty(), &short)
            .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("evaluated value byte"));
}

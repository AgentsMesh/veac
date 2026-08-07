use std::sync::Arc;

use crate::program::expression::{
    evaluate_lookup_with_budget, Environment, ExecutionBudget, ExpressionContext, Value,
};

#[test]
fn fuel_accumulates_across_distinct_evaluations() {
    let values = Environment::new();
    let functions = ExpressionContext::empty();
    let budget = ExecutionBudget::with_limit(2);
    evaluate_lookup_with_budget("1", &values, &functions, &budget).unwrap();
    evaluate_lookup_with_budget("2", &values, &functions, &budget).unwrap();
    let error = evaluate_lookup_with_budget("3", &values, &functions, &budget).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("2 fuel limit"));
}

#[test]
fn evaluated_text_bytes_accumulate_across_evaluations() {
    let values = Environment::new();
    let functions = ExpressionContext::empty();
    let budget = ExecutionBudget::with_limits(usize::MAX, 3);
    evaluate_lookup_with_budget(r#""ab""#, &values, &functions, &budget).unwrap();
    let error = evaluate_lookup_with_budget(r#""cd""#, &values, &functions, &budget).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("evaluated value byte"));
}

#[test]
fn text_value_clones_share_their_payload() {
    let original = Value::Text(Arc::from("large-payload"));
    let cloned = original.clone();
    let (Value::Text(original), Value::Text(cloned)) = (original, cloned) else {
        unreachable!("constructed values are text")
    };
    assert!(Arc::ptr_eq(&original, &cloned));
}

#[test]
fn core_charges_operands_before_the_result_instruction() {
    let budget = ExecutionBudget::with_limit(2);
    let error = evaluate_lookup_with_budget(
        "1 + 2",
        &Environment::new(),
        &ExpressionContext::empty(),
        &budget,
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert_eq!(error.span(), 0..5);
}

#[test]
fn short_circuit_skips_all_unselected_branch_fuel() {
    for source in ["false && (1 / 0 == 0.0)", "true || (1 / 0 == 0.0)"] {
        let budget = ExecutionBudget::with_limit(2);
        evaluate_lookup_with_budget(
            source,
            &Environment::new(),
            &ExpressionContext::empty(),
            &budget,
        )
        .unwrap();
    }
}

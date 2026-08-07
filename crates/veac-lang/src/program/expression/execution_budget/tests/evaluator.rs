use super::super::{ExecutionBudget, LOGICAL_CORE_SLOT_BYTES, MAX_EVALUATOR_STORAGE_BYTES};
use super::unlimited;
use crate::program::expression::{
    compile_expression, compile_functions, runtime, Environment, ExpressionContext,
    FunctionDefinition, FunctionParameter, PrimitiveType, TypeEnvironment, ValueType,
};

fn slot_budget(slots: usize) -> ExecutionBudget {
    let mut limits = unlimited();
    limits.evaluator_storage_bytes = slots * LOGICAL_CORE_SLOT_BYTES;
    ExecutionBudget::with_resource_limits(limits)
}

fn enter_function(
    budget: &ExecutionBudget,
    slots: usize,
    span: std::ops::Range<usize>,
) -> Result<(), super::super::ExpressionError> {
    budget.reserve_evaluator_slots(slots, span)
}

#[test]
fn exact_evaluator_storage_limit_succeeds() {
    slot_budget(2).reserve_evaluator_slots(2, 1..2).unwrap();
}

#[test]
fn one_slot_beyond_evaluator_storage_limit_fails() {
    let error = slot_budget(2).reserve_evaluator_slots(3, 3..5).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("evaluator storage byte"));
    assert_eq!(error.span(), 3..5);
}

#[test]
fn evaluator_slot_multiplication_overflow_is_a_limit_error() {
    let budget = ExecutionBudget::with_resource_limits(unlimited());
    let overflowing = usize::MAX / LOGICAL_CORE_SLOT_BYTES + 1;
    let error = budget
        .reserve_evaluator_slots(overflowing, 5..8)
        .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("evaluator storage byte"));
    assert_eq!(error.span(), 5..8);
}

#[test]
fn failed_evaluator_reservation_does_not_commit() {
    let budget = slot_budget(1);
    budget.reserve_evaluator_slots(2, 0..1).unwrap_err();
    budget.reserve_evaluator_slots(1, 1..2).unwrap();
}

#[test]
fn function_entries_share_one_evaluator_storage_ledger() {
    let budget = slot_budget(3);
    enter_function(&budget, 1, 0..1).unwrap();
    enter_function(&budget, 2, 1..2).unwrap();
    let error = enter_function(&budget, 1, 2..3).unwrap_err();
    assert!(error.message().contains("evaluator storage byte"));
}

#[test]
fn default_evaluator_storage_limit_is_exactly_sixty_four_mibibytes() {
    assert_eq!(MAX_EVALUATOR_STORAGE_BYTES, 64 * 1024 * 1024);
}

#[test]
fn mutable_activation_slots_are_charged_in_addition_to_core_values() {
    let compiled = compile_expression(
        "{ var value = 1; value }",
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap();
    let without_locals = compiled.core().value_count() + compiled.core().inputs().len();
    let error =
        runtime::execute(&compiled, &Environment::new(), &slot_budget(without_locals)).unwrap_err();
    assert!(error.message().contains("evaluator storage byte"));
    runtime::execute(
        &compiled,
        &Environment::new(),
        &slot_budget(without_locals + compiled.core().local_slots().len()),
    )
    .unwrap();
}

#[test]
fn dynamic_function_calls_share_the_evaluator_storage_ledger() {
    let mut definitions = vec![scalar_function("f8", "x + 1.0")];
    for index in (0..8).rev() {
        definitions.push(scalar_function(
            &format!("f{index}"),
            &format!("f{}(x) + f{}(x)", index + 1, index + 1),
        ));
    }
    let context = compile_functions(&ExpressionContext::empty(), &definitions).unwrap();
    let compiled = compile_expression("f0(0.0)", &TypeEnvironment::new(), &context).unwrap();
    let error = runtime::execute(&compiled, &Environment::new(), &slot_budget(64)).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXECUTION_LIMIT");
    assert!(error.message().contains("evaluator storage byte"));
}

fn scalar_function(name: &str, body: &str) -> FunctionDefinition {
    let scalar = ValueType::primitive(PrimitiveType::Scalar);
    FunctionDefinition::new(
        name,
        vec![FunctionParameter::new("x", scalar.clone())],
        scalar,
        format!("{{{body}}}"),
    )
}

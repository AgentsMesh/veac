use super::super::resource::Resource;
use super::super::{
    ExecutionBudget, LOGICAL_CLOSURE_BASE_BYTES, LOGICAL_CLOSURE_CAPTURE_SLOT_BYTES,
    LOGICAL_CLOSURE_VALUE_BYTES, LOGICAL_CORE_SLOT_BYTES,
};
use super::{delta, unlimited};
use crate::program::expression::{
    compile_expression, evaluate, evaluate_in, evaluate_lookup_with_budget, Environment,
    ExpressionContext, TypeEnvironment, Value, MAX_CLOSURE_CAPTURES, MAX_FUNCTION_CALL_DEPTH,
};

#[path = "closure/support.rs"]
mod support;
use support::{capturing_source, depth_functions};

const CAPTURE_SOURCE: &str =
    "{ let first = 20; let second = 22; fn() -> int effect pure { first + second } }";
fn slots(program: &crate::program::expression::CoreProgram) -> usize {
    program.value_count() + program.inputs().len()
}

fn instructions(program: &crate::program::expression::CoreProgram) -> usize {
    program
        .blocks()
        .iter()
        .map(|block| block.instructions().len())
        .sum()
}

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
fn closure_construction_has_exact_handle_and_capture_frame_costs() {
    let Value::Closure(value) = evaluate(CAPTURE_SOURCE, &Environment::new()).unwrap() else {
        panic!("expression must produce a closure");
    };
    let capture_bytes = LOGICAL_CLOSURE_BASE_BYTES + 2 * LOGICAL_CLOSURE_CAPTURE_SLOT_BYTES;
    assert_eq!(value.capture_count(), 2);
    assert_eq!(value.logical_capture_bytes(), capture_bytes);

    let compiled = compile_expression(
        CAPTURE_SOURCE,
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap();
    let program_bytes = slots(compiled.core()) * LOGICAL_CORE_SLOT_BYTES;
    let mut limits = unlimited();
    limits.value_bytes = LOGICAL_CLOSURE_VALUE_BYTES;
    limits.evaluator_storage_bytes = program_bytes + capture_bytes;
    evaluate_with(
        CAPTURE_SOURCE,
        &ExecutionBudget::with_resource_limits(limits),
    )
    .unwrap();
}

#[test]
fn failed_capture_reservations_are_atomic_across_resource_dimensions() {
    let compiled = compile_expression(
        CAPTURE_SOURCE,
        &TypeEnvironment::new(),
        &ExpressionContext::empty(),
    )
    .unwrap();
    let program_bytes = slots(compiled.core()) * LOGICAL_CORE_SLOT_BYTES;
    let capture_bytes = LOGICAL_CLOSURE_BASE_BYTES + 2 * LOGICAL_CLOSURE_CAPTURE_SLOT_BYTES;

    let mut short_value = unlimited();
    short_value.value_bytes = LOGICAL_CLOSURE_VALUE_BYTES - 1;
    short_value.evaluator_storage_bytes = program_bytes + capture_bytes;
    let budget = ExecutionBudget::with_resource_limits(short_value);
    assert!(evaluate_with(CAPTURE_SOURCE, &budget).is_err());
    budget
        .reserve(delta(Resource::EvaluatorStorageBytes, capture_bytes), 1..2)
        .unwrap();

    let mut short_storage = unlimited();
    short_storage.value_bytes = LOGICAL_CLOSURE_VALUE_BYTES;
    short_storage.evaluator_storage_bytes = program_bytes + capture_bytes - 1;
    let budget = ExecutionBudget::with_resource_limits(short_storage);
    assert!(evaluate_with(CAPTURE_SOURCE, &budget).is_err());
    budget
        .reserve(
            delta(Resource::ValueBytes, LOGICAL_CLOSURE_VALUE_BYTES),
            2..3,
        )
        .unwrap();
}

#[test]
fn closure_invocation_shares_fuel_and_evaluator_storage_ledgers() {
    let source = "{ let callback = fn(value: int) -> int effect pure { value + 1 }; callback(41) }";
    let compiled =
        compile_expression(source, &TypeEnvironment::new(), &ExpressionContext::empty()).unwrap();
    let definition = &compiled.core().closure_definitions()[0];
    let fuel = instructions(compiled.core()) + instructions(definition.body());
    let storage = (slots(compiled.core()) + slots(definition.body())) * LOGICAL_CORE_SLOT_BYTES
        + LOGICAL_CLOSURE_BASE_BYTES;

    let mut exact = unlimited();
    exact.fuel = fuel;
    exact.value_bytes = LOGICAL_CLOSURE_VALUE_BYTES;
    exact.evaluator_storage_bytes = storage;
    assert_eq!(
        evaluate_with(source, &ExecutionBudget::with_resource_limits(exact)).unwrap(),
        Value::Integer(42)
    );

    for (resource, limit) in [
        (Resource::Fuel, fuel - 1),
        (Resource::EvaluatorStorageBytes, storage - 1),
    ] {
        let mut limits = exact;
        super::set_limit(&mut limits, resource, limit);
        let error =
            evaluate_with(source, &ExecutionBudget::with_resource_limits(limits)).unwrap_err();
        assert!(error.starts_with("EXPRESSION_EXECUTION_LIMIT:"));
    }
}

#[test]
fn closure_invocation_uses_the_same_call_depth_as_named_functions() {
    let exact = depth_functions(MAX_FUNCTION_CALL_DEPTH - 1);
    let value = evaluate_in(
        "f0(fn() -> int effect pure { 42 })",
        &Environment::new(),
        &exact,
    )
    .unwrap();
    assert_eq!(value, Value::Integer(42));

    let excessive = depth_functions(MAX_FUNCTION_CALL_DEPTH);
    let error = evaluate_in(
        "f0(fn() -> int effect pure { 42 })",
        &Environment::new(),
        &excessive,
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_CALL_DEPTH_LIMIT");
}

#[test]
fn capture_count_is_bounded_before_construction() {
    assert!(evaluate(&capturing_source(MAX_CLOSURE_CAPTURES), &Environment::new()).is_ok());
    let source = capturing_source(MAX_CLOSURE_CAPTURES + 1);
    let error = evaluate(&source, &Environment::new()).unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_CLOSURE_CAPTURE_LIMIT");
}

#[test]
fn external_environment_function_values_are_rejected_before_execution() {
    let closure = evaluate("fn() -> int effect pure { 42 }", &Environment::new()).unwrap();
    let mut environment = Environment::new();
    environment.insert("external".to_owned(), closure);
    let budget = ExecutionBudget::with_resource_limits(unlimited());
    let error = evaluate_lookup_with_budget(
        "external()",
        &environment,
        &ExpressionContext::empty(),
        &budget,
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_EXTERNAL_FUNCTION_VALUE");
}

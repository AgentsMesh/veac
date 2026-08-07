use std::collections::BTreeMap;
use std::sync::Arc;

use super::super::metadata::EffectEvidence;
use crate::program::expression::{
    evaluate, CompiledExpression, Effect, Environment, ExecutionBudget, ExpressionContext,
    FunctionMap, Value,
};

type Values = BTreeMap<String, Arc<Value>>;

fn closure(source: &str) -> Value {
    evaluate(source, &Environment::new()).unwrap()
}

fn compile(values: &Values) -> CompiledExpression {
    crate::program::expression::compile::compile_with_values(
        "callback(41)",
        values,
        &ExpressionContext::empty(),
    )
    .unwrap()
}

#[test]
fn trusted_callable_input_is_self_describing_and_executes_verified_code() {
    let value = closure("{ let offset = 1; fn(value: int) -> int effect pure { value + offset } }");
    let Value::Closure(expected) = &value else {
        unreachable!()
    };
    let expected = Arc::clone(expected);
    let values = Values::from([("callback".to_owned(), Arc::new(value))]);
    let compiled = compile(&values);
    let input = &compiled.core().inputs()[0];
    let contract = input.callable().unwrap();
    assert_eq!(contract.definition_digest(), expected.definition_digest());
    assert_eq!(contract.summary(), expected.summary());
    assert_eq!(contract.capture_types(), expected.capture_types());
    assert_eq!(
        crate::program::expression::runtime::execute(
            &compiled,
            &values,
            &ExecutionBudget::default(),
        )
        .unwrap(),
        Value::Integer(42)
    );
}

#[test]
fn forged_callable_summary_is_rejected_by_core_metadata_verification() {
    let value = closure("fn(value: int) -> int effect pure { value + 1 }");
    let values = Values::from([("callback".to_owned(), Arc::new(value))]);
    let compiled = compile(&values);
    let mut program = compiled.core().clone();
    program.inputs[0].callable.as_mut().unwrap().summary.effect =
        EffectEvidence::from_effect(Effect::GraphEmit);
    let functions = FunctionMap::new();
    let error =
        super::super::verify_with_input_trust(program, functions.registry(), &[], &|name| {
            name == "callback"
        })
        .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_CORE_VERIFY");
    assert_eq!(
        error.message(),
        "instruction metadata does not match its operands"
    );
}

#[test]
fn runtime_rejects_a_different_closure_before_invocation() {
    let first = closure("fn(value: int) -> int effect pure { value + 1 }");
    let mut values = Values::from([("callback".to_owned(), Arc::new(first))]);
    let compiled = compile(&values);
    let second = closure("fn(value: int) -> int effect pure { value + 2 }");
    values.insert("callback".to_owned(), Arc::new(second));
    let error = crate::program::expression::runtime::execute(
        &compiled,
        &values,
        &ExecutionBudget::default(),
    )
    .unwrap_err();
    assert_eq!(error.code(), "EXPRESSION_RUNTIME_CONTRACT");
    assert_eq!(&"callback(41)"[error.span()], "callback");
}

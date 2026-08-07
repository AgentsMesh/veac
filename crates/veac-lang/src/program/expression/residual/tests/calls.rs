use std::collections::BTreeMap;

use super::support::*;
use crate::program::expression::{
    compile_functions, compile_temporal_expression, ExpressionContext, FunctionDefinition,
    FunctionParameter, PrimitiveType, ResidualRuntimeValue, TypeEnvironment, Value, ValueType,
};
use veac_ir::{TemporalType, TemporalValue};

fn scalar(name: &str) -> FunctionParameter {
    FunctionParameter::new(name, ValueType::primitive(PrimitiveType::Scalar))
}

fn function(name: &str, body: &str) -> FunctionDefinition {
    FunctionDefinition::new(
        name,
        vec![scalar("value")],
        ValueType::primitive(PrimitiveType::Scalar),
        format!("{{ {body} }}"),
    )
}

fn compiled(source: &str, definitions: &[FunctionDefinition]) -> super::super::CompiledExpression {
    let context = compile_functions(&ExpressionContext::empty(), definitions).unwrap();
    let inputs = BTreeMap::from([(
        "progress".to_owned(),
        parameter("progress", TemporalType::Scalar),
    )]);
    compile_temporal_expression(source, &TypeEnvironment::new(), &inputs, &context).unwrap()
}

#[test]
fn pure_nested_user_calls_residualize_through_verified_core() {
    let definitions = [
        function("bounded", "clamp(value, 0.0, 1.0)"),
        function("pulse", "bounded(value * 2.0)"),
    ];
    let result = residualize(
        &compiled("pulse(progress)", &definitions),
        &no_bindings(),
        "nested_calls",
    );
    assert_eq!(
        evaluate(&result, vec![(0, TemporalValue::Scalar { value: 0.3 })]),
        TemporalValue::Scalar { value: 0.6 }
    );
}

#[test]
fn concrete_and_temporal_extrema_keep_one_semantics() {
    let concrete = residualize(
        &compile("min(4, max(2, 3))", &[]),
        &no_bindings(),
        "extrema",
    );
    assert!(matches!(
        concrete.value(),
        ResidualRuntimeValue::Concrete(Value::Integer(3))
    ));

    let input = parameter("x", TemporalType::Scalar);
    for (name, source, expected) in [
        ("minimum", "min(x, 0.5)", 0.5),
        ("maximum", "max(x, 0.5)", 0.75),
    ] {
        let result = residualize(
            &compile(source, &[("x", input.clone())]),
            &no_bindings(),
            name,
        );
        assert_eq!(
            evaluate(&result, vec![(0, TemporalValue::Scalar { value: 0.75 })]),
            TemporalValue::Scalar { value: expected }
        );
    }
}

#[test]
fn temporal_clamp_requires_ordered_build_stage_bounds() {
    let input = parameter("x", TemporalType::Scalar);
    let dynamic = compile("clamp(x, x, 1.0)", &[("x", input.clone())]);
    let error =
        super::super::residualize_expression(&dynamic, &no_bindings(), request("dynamic_bounds"))
            .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_CLAMP_DYNAMIC_RANGE");

    let inverted = compile("clamp(x, 1.0, 0.0)", &[("x", input)]);
    let error =
        super::super::residualize_expression(&inverted, &no_bindings(), request("inverted_bounds"))
            .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_CLAMP_RANGE");
}

#[test]
fn graph_emitting_user_function_is_not_a_temporal_escape_hatch() {
    let effectful = function(
        "effectful",
        "let emitted = item(identifier(\"emitted\"), item_enabled(), during(0s, 1s), \
         source_generated(generator_transparent()), source_timing_native()); value",
    );
    let expression = compiled("effectful(progress)", &[effectful]);
    let error = super::super::residualize_expression(
        &expression,
        &no_bindings(),
        request("effectful_call"),
    )
    .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_EFFECT_UNSUPPORTED");
}

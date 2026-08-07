use super::support::*;
use crate::program::expression::{PrimitiveType, ResidualRuntimeValue, TypeEnvironment, Value};
use veac_ir::{TemporalNodeKind, TemporalType, TemporalValue};

fn branch_expression() -> crate::program::expression::CompiledExpression {
    let build =
        TypeEnvironment::from([("choose_dynamic".to_owned(), PrimitiveType::Boolean.into())]);
    compile_with_build(
        "if choose_dynamic { gain } else { 2.0 }",
        build,
        &[("gain", parameter("gain", TemporalType::Scalar))],
    )
}

#[test]
fn concrete_condition_executes_only_the_selected_arm() {
    let expression = branch_expression();
    let concrete = residualize(
        &expression,
        &binding("choose_dynamic", Value::Bool(false)),
        "build_false",
    );
    assert_eq!(
        concrete.value(),
        &ResidualRuntimeValue::Concrete(Value::Scalar(
            crate::program::expression::ExactNumber::integer(2)
        ))
    );
    assert!(concrete.program().is_none());

    let dynamic = residualize(
        &expression,
        &binding("choose_dynamic", Value::Bool(true)),
        "build_true",
    );
    assert!(dynamic.program().is_some());
    assert_eq!(dynamic.inputs().len(), 1);
}

#[test]
fn temporal_condition_generates_a_lazy_select() {
    let expression = compile(
        "if condition { 1.0 } else { 1.0 / divisor }",
        &[
            ("condition", parameter("condition", TemporalType::Boolean)),
            ("divisor", parameter("divisor", TemporalType::Scalar)),
        ],
    );
    let result = residualize(&expression, &no_bindings(), "select");
    assert!(result
        .program()
        .unwrap()
        .nodes
        .iter()
        .any(|node| matches!(node.kind, TemporalNodeKind::Select { .. })));
    assert_eq!(
        evaluate(
            &result,
            vec![
                (0, TemporalValue::Boolean { value: true }),
                (1, TemporalValue::Scalar { value: 0.0 }),
            ],
        ),
        TemporalValue::Scalar { value: 1.0 }
    );
}

#[test]
fn temporal_branch_rejects_graph_effect_before_executing_an_arm() {
    let expression = compile(
        "if condition { project(identifier(\"a\"), project_settings(600)) } \
         else { project(identifier(\"b\"), project_settings(600)) }",
        &[("condition", parameter("condition", TemporalType::Boolean))],
    );
    let error = super::super::residualize_expression(
        &expression,
        &no_bindings(),
        request("effectful_branch"),
    )
    .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_TEMPORAL_BRANCH_EFFECT");
    assert!(error.message().contains("provably Pure"));
    assert!(error.span().end > error.span().start);
}

#[test]
fn pure_structural_branch_result_fails_closed() {
    let expression = compile(
        "if condition { 0..1 } else { 1..2 }",
        &[("condition", parameter("condition", TemporalType::Boolean))],
    );
    let error = super::super::residualize_expression(
        &expression,
        &no_bindings(),
        request("unsupported_branch"),
    )
    .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_VALUE_UNSUPPORTED");
}

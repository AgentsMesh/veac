use std::sync::Arc;

use super::support::*;
use crate::program::expression::{
    CoreBuildInputId, CoreInstruction, CoreInstructionKind, CoreTypeId, CoreValueMetadata, Effect,
    PrimitiveType, ResidualBuildBindings, ResidualizationLimits, TypeEnvironment, Value, ValueId,
};
use veac_ir::{SequenceId, TemporalType};

#[test]
fn build_bindings_are_required_and_exactly_typed() {
    let build = TypeEnvironment::from([("gain".to_owned(), PrimitiveType::Scalar.into())]);
    let expression = compile_with_build("gain + 1.0", build, &[]);
    let missing =
        super::super::residualize_expression(&expression, &no_bindings(), request("missing"))
            .unwrap_err();
    assert_eq!(missing.code(), "RESIDUAL_BUILD_INPUT_MISSING");

    let wrong = super::super::residualize_expression(
        &expression,
        &binding("gain", Value::Integer(1)),
        request("wrong_type"),
    )
    .unwrap_err();
    assert_eq!(wrong.code(), "RESIDUAL_BUILD_INPUT_TYPE");
    assert!(wrong.message().contains("gain"));
}

#[test]
fn step_node_and_value_budgets_fail_at_the_boundary() {
    let input = parameter("x", TemporalType::Scalar);
    let expression = compile("x", &[("x", input)]);
    for (field, code) in [
        ("steps", "RESIDUAL_STEP_LIMIT"),
        ("nodes", "RESIDUAL_NODE_LIMIT"),
    ] {
        let mut request = request(field);
        if field == "steps" {
            request.limits.max_steps = 0;
        } else {
            request.limits.max_nodes = 0;
        }
        let error =
            super::super::residualize_expression(&expression, &no_bindings(), request).unwrap_err();
        assert_eq!(error.code(), code);
    }

    let text = compile(
        "if condition { \"a\" } else { \"b\" }",
        &[("condition", parameter("condition", TemporalType::Boolean))],
    );
    let mut request = request("value_limit");
    request.limits.max_value_bytes = 0;
    let error = super::super::residualize_expression(&text, &no_bindings(), request).unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_VALUE_LIMIT");
}

#[test]
fn failure_does_not_leak_partial_builder_state_into_a_retry() {
    let expression = compile("x + 1.0", &[("x", parameter("x", TemporalType::Scalar))]);
    let mut limited = request("rollback_limited");
    limited.limits = ResidualizationLimits {
        max_nodes: 1,
        ..ResidualizationLimits::default()
    };
    assert_eq!(
        super::super::residualize_expression(&expression, &no_bindings(), limited)
            .unwrap_err()
            .code(),
        "RESIDUAL_NODE_LIMIT"
    );
    let retried = residualize(&expression, &no_bindings(), "rollback_retry");
    assert_eq!(retried.program().unwrap().nodes[0].id.get(), 0);
}

#[test]
fn invalid_provenance_is_rejected_before_publication() {
    let expression = compile("1", &[]);
    let mut request = request("bad_provenance");
    request.provenance.definition.name.clear();
    let error =
        super::super::residualize_expression(&expression, &no_bindings(), request).unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_TEMPORAL_VALIDATE");
    assert!(error.message().contains("TEMPORAL_DEFINITION_NAME"));
}

#[test]
fn clock_sources_with_different_owners_never_alias_silently() {
    let inputs = [
        (
            "first",
            crate::program::expression::CoreTemporalInputIdentity::SequenceTime {
                sequence_id: SequenceId::new("seq_first").unwrap(),
            },
        ),
        (
            "second",
            crate::program::expression::CoreTemporalInputIdentity::SequenceTime {
                sequence_id: SequenceId::new("seq_second").unwrap(),
            },
        ),
    ];
    let expression = compile("first + second", &inputs);
    let error =
        super::super::residualize_expression(&expression, &no_bindings(), request("clock_owner"))
            .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_INPUT_SOURCE_DUPLICATE");
}

#[test]
fn unsupported_concrete_values_fail_with_source_span() {
    let build = TypeEnvironment::from([
        ("first".to_owned(), PrimitiveType::Identifier.into()),
        ("second".to_owned(), PrimitiveType::Identifier.into()),
    ]);
    let expression = compile_with_build(
        "if condition { first } else { second }",
        build,
        &[("condition", parameter("condition", TemporalType::Boolean))],
    );
    let bindings = ResidualBuildBindings::from([
        (
            CoreBuildInputId::for_symbol("first"),
            Value::Identifier(Arc::from("first")),
        ),
        (
            CoreBuildInputId::for_symbol("second"),
            Value::Identifier(Arc::from("second")),
        ),
    ]);
    let error = super::super::residualize_expression(&expression, &bindings, request("identifier"))
        .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_VALUE_UNSUPPORTED");
    assert!(error.span().end > error.span().start);
}

#[test]
fn local_mutation_metadata_is_rejected_even_without_graph_emission() {
    let mut metadata = CoreValueMetadata::constant();
    metadata.effect =
        crate::program::expression::core::EffectEvidence::from_effect(Effect::LocalMutation);
    let instruction = CoreInstruction {
        id: ValueId::new(0),
        kind: CoreInstructionKind::Literal(Value::Integer(1)),
        type_id: CoreTypeId::new(0),
        metadata,
        span: 12..17,
    };
    let error = super::super::effect::instruction(&instruction).unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_EFFECT_UNSUPPORTED");
    assert_eq!(error.span(), 12..17);
}

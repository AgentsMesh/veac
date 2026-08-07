use std::collections::BTreeMap;

use veac_ir::{
    evaluate_temporal_program, TemporalAuthoredSite, TemporalDefinitionId, TemporalDefinitionKind,
    TemporalDefinitionSite, TemporalEvaluationInput, TemporalEvaluationLimits, TemporalLogicalKey,
    TemporalParameterId, TemporalProgramId, TemporalProvenance, TemporalProvenanceId,
    TemporalSourceId, TemporalSourceSpan, TemporalType, TemporalValue,
};
use veac_lang::program::expression::{
    compile_temporal_expression, residualize_expression, CoreTemporalInputIdentity,
    ExpressionContext, ResidualBuildBindings, ResidualizationLimits, ResidualizationRequest,
    TypeEnvironment,
};

fn provenance(name: &str) -> TemporalProvenance {
    let definition_id = TemporalDefinitionId::new(format!("def_{name}")).unwrap();
    let source_id = TemporalSourceId::new(format!("src_{name}")).unwrap();
    let span = TemporalSourceSpan { start: 0, end: 10 };
    TemporalProvenance {
        id: TemporalProvenanceId::new(format!("tpv_{name}")).unwrap(),
        definition: TemporalDefinitionSite {
            id: definition_id.clone(),
            kind: TemporalDefinitionKind::Function,
            name: name.into(),
            source_id: source_id.clone(),
            span,
        },
        origin: TemporalAuthoredSite {
            definition_id,
            function: name.into(),
            source_id,
            span,
        },
        call_stack: Vec::new(),
        logical_keys: vec![TemporalLogicalKey::new(format!("key_{name}")).unwrap()],
    }
}

fn compile(
    source: &str,
    inputs: &[(&str, TemporalType)],
) -> veac_lang::program::expression::CompiledExpression {
    let temporal_inputs = inputs
        .iter()
        .map(|(name, value_type)| {
            (
                (*name).to_owned(),
                CoreTemporalInputIdentity::Parameter {
                    parameter_id: TemporalParameterId::new(format!("tpm_{name}")).unwrap(),
                    value_type: *value_type,
                },
            )
        })
        .collect::<BTreeMap<_, _>>();
    compile_temporal_expression(
        source,
        &TypeEnvironment::new(),
        &temporal_inputs,
        &ExpressionContext::empty(),
    )
    .unwrap()
}

fn residualize(
    expression: &veac_lang::program::expression::CompiledExpression,
    name: &str,
) -> veac_lang::program::expression::ResidualizedExpression {
    residualize_expression(
        expression,
        &ResidualBuildBindings::new(),
        ResidualizationRequest {
            program_id: TemporalProgramId::new(format!("tpg_{name}")).unwrap(),
            provenance: provenance(name),
            limits: ResidualizationLimits::default(),
        },
    )
    .unwrap()
}

#[test]
fn public_compile_residualize_evaluate_pipeline_is_typed_and_deterministic() {
    let expression = compile(
        "if enabled { gain * 2.0 } else { gain / 2.0 }",
        &[
            ("enabled", TemporalType::Boolean),
            ("gain", TemporalType::Scalar),
        ],
    );
    let result = residualize(&expression, "pipeline");
    let program = result.program().unwrap();
    let digest = program.content_sha256.clone();
    for (enabled, gain, expected) in [(true, 3.0, 6.0), (false, 3.0, 1.5)] {
        let actual = evaluate_temporal_program(
            program,
            &[
                TemporalEvaluationInput {
                    input_id: veac_ir::TemporalInputId::new(0),
                    value: TemporalValue::Boolean { value: enabled },
                },
                TemporalEvaluationInput {
                    input_id: veac_ir::TemporalInputId::new(1),
                    value: TemporalValue::Scalar { value: gain },
                },
            ],
            TemporalEvaluationLimits::default(),
        )
        .unwrap();
        assert_eq!(actual, TemporalValue::Scalar { value: expected });
        assert_eq!(program.content_sha256, digest);
    }
    assert_eq!(result.inputs().len(), 2);
}

#[test]
fn public_pipeline_fails_closed_on_graph_emission() {
    let expression = compile("project(identifier(\"demo\"), project_settings(600))", &[]);
    let error = residualize_expression(
        &expression,
        &ResidualBuildBindings::new(),
        ResidualizationRequest {
            program_id: TemporalProgramId::new("tpg_graph").unwrap(),
            provenance: provenance("graph"),
            limits: ResidualizationLimits::default(),
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_EFFECT_UNSUPPORTED");
}

#[test]
fn public_pipeline_fails_closed_on_compiled_local_mutation() {
    let expression = compile("{ var value = 1; set value = value + 1; value }", &[]);
    let error = residualize_expression(
        &expression,
        &ResidualBuildBindings::new(),
        ResidualizationRequest {
            program_id: TemporalProgramId::new("tpg_local_mutation").unwrap(),
            provenance: provenance("local_mutation"),
            limits: ResidualizationLimits::default(),
        },
    )
    .unwrap_err();
    assert_eq!(error.code(), "RESIDUAL_EFFECT_UNSUPPORTED");
}

#[test]
fn temporal_builtins_ordering_and_equality_residualize_and_execute() {
    let cases = [
        ("min(value, 2.0)", 3.0, TemporalValue::Scalar { value: 2.0 }),
        ("max(value, 2.0)", 1.0, TemporalValue::Scalar { value: 2.0 }),
        (
            "clamp(value, 1.0, 3.0)",
            4.0,
            TemporalValue::Scalar { value: 3.0 },
        ),
        ("value < 2.0", 1.0, TemporalValue::Boolean { value: true }),
        ("value == 2.0", 2.0, TemporalValue::Boolean { value: true }),
    ];
    for (index, (source, value, expected)) in cases.into_iter().enumerate() {
        let expression = compile(source, &[("value", TemporalType::Scalar)]);
        let result = residualize(&expression, &format!("operator_{index}"));
        let actual = evaluate_temporal_program(
            result.program().unwrap(),
            &[TemporalEvaluationInput {
                input_id: veac_ir::TemporalInputId::new(0),
                value: TemporalValue::Scalar { value },
            }],
            TemporalEvaluationLimits::default(),
        )
        .unwrap();
        assert_eq!(actual, expected);
    }

    for (source, expected) in [
        ("1 < 2", veac_lang::program::expression::Value::Bool(true)),
        ("2 == 3", veac_lang::program::expression::Value::Bool(false)),
    ] {
        let expression = compile(source, &[]);
        let result = residualize(&expression, "constant_compare");
        assert_eq!(
            result.value(),
            &veac_lang::program::expression::ResidualRuntimeValue::Concrete(expected)
        );
    }
}

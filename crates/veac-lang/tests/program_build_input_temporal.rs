use veac_ir::{TemporalEvaluationInput, TemporalEvaluationLimits, TemporalValue};
use veac_lang::program::{
    prepare_source, BuildInputBinding, BuildInputManifestV1, BuildInputManifestValue,
};

#[allow(dead_code)]
#[path = "executable_authored_temporal/support.rs"]
mod support;

#[test]
fn declared_build_analysis_input_is_concrete_before_temporal_residualization() {
    let source = format!("input analysis amplitude: scalar;\n{}", support::MAIN)
        .replace("pulse(progress)", "pulse(progress) * amplitude");
    let mut inputs = BuildInputManifestV1::empty();
    inputs.inputs.push(BuildInputBinding {
        name: "amplitude".into(),
        value: BuildInputManifestValue::Scalar {
            value: "0.5".into(),
        },
    });
    let built = prepare_source(&source)
        .unwrap()
        .execute_with_inputs(&inputs)
        .unwrap();
    let envelope = built.envelope();
    let binding = &envelope.temporal.bindings[0];
    assert_eq!(binding.clocks.len(), 1);
    assert!(binding.parameters.is_empty());
    assert!(!serde_json::to_string(envelope)
        .unwrap()
        .contains("\"analyses\""));
    let value = veac_ir::evaluate_temporal_program(
        &envelope.temporal.programs[0],
        &[TemporalEvaluationInput {
            input_id: binding.clocks[0].input_id,
            value: TemporalValue::Scalar { value: 0.25 },
        }],
        TemporalEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(value, TemporalValue::Scalar { value: 0.25 });
}

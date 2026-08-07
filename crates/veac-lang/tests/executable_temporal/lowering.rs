use veac_ir::{
    evaluate_temporal_program, Animatable, TemporalClock, TemporalClockOwner,
    TemporalEvaluationInput, TemporalEvaluationLimits, TemporalParameterId, TemporalType,
    TemporalValue,
};
use veac_lang::program::expression::{
    compile_temporal_expression, CoreBuildInputId, CoreTemporalInputIdentity, ExactNumber,
    ExpressionContext, PrimitiveType, TypeEnvironment, Value, ValueType,
};
use veac_lang::program::ClipTemporalProperty;

use super::support::{self, SOLID_SOURCE};

#[test]
fn temporal_leaf_publishes_executable_ir_and_attaches_to_the_closed_sink() {
    let ids = support::ids(SOLID_SOURCE);
    let gain_id = TemporalParameterId::new("tpm_gain").unwrap();
    let expression = support::compile(
        "progress * gain",
        [
            (
                "progress",
                CoreTemporalInputIdentity::Progress {
                    item_id: ids.items[0].clone(),
                },
            ),
            (
                "gain",
                CoreTemporalInputIdentity::Parameter {
                    parameter_id: gain_id.clone(),
                    value_type: TemporalType::Scalar,
                },
            ),
        ],
    );
    let leaf = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression,
        "opacity",
    )
    .bind_parameter(gain_id.clone(), TemporalValue::Scalar { value: 0.8 });
    let envelope = support::execute(SOLID_SOURCE, &[leaf]);
    let clip = &envelope.project.sequences[0].tracks[0].clips[0];
    let binding_id = match &clip.visual.as_ref().unwrap().opacity {
        Animatable::Binding { binding_id } => binding_id,
        value => panic!("expected canonical binding, got {value:?}"),
    };
    let library = &envelope.temporal;
    assert_eq!((library.programs.len(), library.bindings.len()), (1, 1));
    assert_eq!(library.provenance.len(), 1);
    let binding = &library.bindings[0];
    assert_eq!(&binding.id, binding_id);
    assert_eq!(binding.result_type, TemporalType::Scalar);
    assert_eq!(binding.clocks[0].clock, TemporalClock::Progress);
    assert_eq!(
        binding.clocks[0].owner,
        TemporalClockOwner::Item {
            item_id: ids.items[0].clone(),
        }
    );
    assert_eq!(binding.parameters[0].parameter_id, gain_id);
    assert_eq!(
        binding.parameters[0].value,
        TemporalValue::Scalar { value: 0.8 }
    );

    let value = evaluate_temporal_program(
        &library.programs[0],
        &[
            TemporalEvaluationInput {
                input_id: binding.clocks[0].input_id,
                value: TemporalValue::Scalar { value: 0.5 },
            },
            TemporalEvaluationInput {
                input_id: binding.parameters[0].input_id,
                value: binding.parameters[0].value.clone(),
            },
        ],
        TemporalEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(value, TemporalValue::Scalar { value: 0.4 });
    assert!(veac_ir::validate(&envelope).is_ok());
    let json = veac_ir::canonical_json(&envelope).unwrap();
    assert_eq!(veac_ir::decode_canonical_json(&json).unwrap(), envelope);
}

#[test]
fn equal_residual_programs_are_content_pooled_across_clip_owners() {
    let ids = support::ids(SOLID_SOURCE);
    let leaves = ids
        .items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let expression = support::compile(
                "progress",
                [(
                    "progress",
                    CoreTemporalInputIdentity::Progress {
                        item_id: item.clone(),
                    },
                )],
            );
            support::leaf(
                item.clone(),
                ClipTemporalProperty::VisualOpacity,
                expression,
                &format!("pool_{index}"),
            )
        })
        .collect::<Vec<_>>();
    let envelope = support::execute(SOLID_SOURCE, &leaves);
    assert_eq!(envelope.temporal.programs.len(), 1);
    assert_eq!(envelope.temporal.bindings.len(), 2);
    let program_id = &envelope.temporal.programs[0].id;
    assert!(envelope
        .temporal
        .bindings
        .iter()
        .all(|binding| &binding.program_id == program_id));
    assert_ne!(
        envelope.temporal.bindings[0].provenance_id,
        envelope.temporal.bindings[1].provenance_id
    );
    assert!(veac_ir::validate(&envelope).is_ok());
}

#[test]
fn known_build_inputs_fold_before_the_temporal_program_is_published() {
    let ids = support::ids(SOLID_SOURCE);
    let build = TypeEnvironment::from([(
        "gain".to_owned(),
        ValueType::primitive(PrimitiveType::Scalar),
    )]);
    let temporal = [(
        "progress".to_owned(),
        CoreTemporalInputIdentity::Progress {
            item_id: ids.items[0].clone(),
        },
    )]
    .into_iter()
    .collect();
    let expression = compile_temporal_expression(
        "progress * gain",
        &build,
        &temporal,
        &ExpressionContext::empty(),
    )
    .unwrap();
    let leaf = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression.clone(),
        "build_fold",
    )
    .bind_build(
        CoreBuildInputId::for_symbol("gain"),
        Value::Scalar(ExactNumber::new(1, 2).unwrap()),
    );
    let envelope = support::execute(SOLID_SOURCE, &[leaf]);
    let binding = &envelope.temporal.bindings[0];
    assert!(binding.parameters.is_empty());
    let value = evaluate_temporal_program(
        &envelope.temporal.programs[0],
        &[TemporalEvaluationInput {
            input_id: binding.clocks[0].input_id,
            value: TemporalValue::Scalar { value: 0.5 },
        }],
        TemporalEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(value, TemporalValue::Scalar { value: 0.25 });

    let missing = support::leaf(
        ids.items[0].clone(),
        ClipTemporalProperty::VisualOpacity,
        expression,
        "build_missing",
    );
    let diagnostic = support::failure(SOLID_SOURCE, &[missing]);
    assert!(diagnostic.message.contains("RESIDUAL_BUILD_INPUT_MISSING"));
}

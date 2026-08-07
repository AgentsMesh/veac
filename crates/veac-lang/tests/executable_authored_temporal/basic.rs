use veac_ir::{Animatable, TemporalClock, TemporalValue};
use veac_lang::program::{build_source, prepare_source};

use super::support::{evaluate, MAIN};

#[test]
fn authored_animation_compiles_through_build_ir_and_temporal_evaluation() {
    let built = build_source(MAIN).unwrap();
    let envelope = built.envelope();
    let clip = &envelope.project.sequences[0].tracks[0].clips[0];
    let Animatable::Binding { binding_id } = &clip.visual.as_ref().unwrap().opacity else {
        panic!("authored opacity must lower to a temporal binding");
    };
    let binding = envelope
        .temporal
        .bindings
        .iter()
        .find(|value| &value.id == binding_id)
        .unwrap();
    assert_eq!(binding.clocks.len(), 1);
    assert_eq!(binding.clocks[0].clock, TemporalClock::Progress);
    assert_eq!(
        evaluate(envelope, TemporalValue::Scalar { value: 0.25 }),
        TemporalValue::Scalar { value: 0.5 }
    );
    assert!(veac_ir::validate(envelope).is_ok());
}

#[test]
fn authored_animation_is_part_of_declared_input_and_source_identity() {
    let first = build_source(MAIN).unwrap();
    let second = build_source(&MAIN.replace("pulse(progress)", "pulse(clip_time / 2s)")).unwrap();
    assert_ne!(
        first.envelope().executable.digests.declared_inputs_sha256,
        second.envelope().executable.digests.declared_inputs_sha256
    );
    assert_ne!(
        first.envelope().executable.digests.source_graph_sha256,
        second.envelope().executable.digests.source_graph_sha256
    );
}

#[test]
fn nested_pure_helpers_and_temporal_branches_stay_executable() {
    let source = MAIN.replace(
        "fn pulse(value: scalar) -> scalar { clamp(value * 2.0, 0.0, 1.0) }",
        "fn bound(value: scalar) -> scalar { clamp(value, 0.0, 1.0) }\n\
         fn pulse(value: scalar) -> scalar {\n\
           if value < 0.5 { bound(value * 2.0) } else { bound(2.0 - value * 2.0) }\n\
         }",
    );
    let built = build_source(&source).unwrap();
    assert_eq!(
        evaluate(built.envelope(), TemporalValue::Scalar { value: 0.75 }),
        TemporalValue::Scalar { value: 0.5 }
    );
}

#[test]
fn temporal_expression_must_retain_a_dynamic_input() {
    let source = MAIN.replace("pulse(progress)", "0.5");
    let error = build_source(&source).unwrap_err();
    assert!(error.as_slice()[0]
        .message
        .contains("EXECUTABLE_TEMPORAL_CONCRETE"));
}

#[test]
fn source_time_is_not_implicitly_available() {
    let source = MAIN.replace("pulse(progress)", "source_time / 2s");
    let error = prepare_source(&source).unwrap_err();
    assert_eq!(error.as_slice()[0].code, "PROGRAM_TEMPORAL_EXPRESSION");
    assert!(error.as_slice()[0].message.contains("source_time"));
}

#[test]
fn entry_context_does_not_need_a_dummy_source_reference() {
    assert!(build_source(MAIN).is_ok());
}

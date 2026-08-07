use std::collections::BTreeSet;

use veac_ir::{Animatable, TemporalEvaluationInput, TemporalEvaluationLimits, TemporalValue};
use veac_lang::program::build_source;

#[path = "executable_component_temporal/errors.rs"]
mod errors;
#[path = "executable_component_temporal/iteration.rs"]
mod iteration;
#[path = "executable_component_temporal/modules.rs"]
mod modules;
#[path = "executable_component_temporal/sink_matrix.rs"]
mod sink_matrix;
#[path = "executable_component_temporal/source_time.rs"]
mod source_time;

const SOURCE: &str = r#"
fn pulse(value: scalar) -> scalar { clamp(value * 2.0, 0.0, 1.0) }

fn card(key: identifier, at: time, color: color) -> Item {
  let value = item(key, item_enabled(), during(at, 1s),
    source_generated(generator_solid(color)), source_timing_native());
  animate visual-opacity on clip(value) { pulse(progress) }
}

fn main(context: Context) -> Project {
  let first = card(identifier("first"), 0s, #c43a69ff);
  let second = card(identifier("second"), 1s, #2d8f85ff);
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(first).with_item(second);
  let timeline = sequence(identifier("main"), "组件动画",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("component-temporal"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn one_factory_definition_attaches_two_independent_temporal_bindings() {
    let built = build_source(SOURCE).unwrap();
    let envelope = built.envelope();
    let clips = &envelope.project.sequences[0].tracks[0].clips;
    assert_eq!(envelope.temporal.programs.len(), 1);
    assert_eq!(envelope.temporal.bindings.len(), 2);
    let ids = clips
        .iter()
        .map(|clip| match &clip.visual.as_ref().unwrap().opacity {
            Animatable::Binding { binding_id } => binding_id.clone(),
            _ => panic!("component opacity must be a Temporal binding"),
        })
        .collect::<BTreeSet<_>>();
    assert_eq!(ids.len(), 2);
    for id in ids {
        assert_eq!(evaluate(envelope, &id, 0.25), 0.5);
    }
    veac_ir::validate(envelope).unwrap();
}

#[test]
fn repeated_builds_keep_all_component_temporal_identities_stable() {
    let first = build_source(SOURCE).unwrap();
    let second = build_source(SOURCE).unwrap();
    assert_eq!(first.envelope().temporal, second.envelope().temporal);
}

#[test]
fn animation_source_change_rotates_binding_and_program_identity() {
    let first = build_source(SOURCE).unwrap();
    let changed = SOURCE.replace("value * 2.0", "value * 3.0");
    let second = build_source(&changed).unwrap();
    let first = &first.envelope().temporal;
    let second = &second.envelope().temporal;
    assert_ne!(binding_ids(first), binding_ids(second));
    assert_ne!(program_ids(first), program_ids(second));
    assert_ne!(first.provenance, second.provenance);
}

fn evaluate(envelope: &veac_ir::ProjectEnvelope, id: &veac_ir::TemporalBindingId, x: f64) -> f64 {
    let binding = envelope
        .temporal
        .bindings
        .iter()
        .find(|value| &value.id == id)
        .unwrap();
    let program = envelope
        .temporal
        .programs
        .iter()
        .find(|value| value.id == binding.program_id)
        .unwrap();
    let values = binding
        .clocks
        .iter()
        .map(|clock| TemporalEvaluationInput {
            input_id: clock.input_id,
            value: TemporalValue::Scalar { value: x },
        })
        .collect::<Vec<_>>();
    match veac_ir::evaluate_temporal_program(program, &values, TemporalEvaluationLimits::default())
        .unwrap()
    {
        TemporalValue::Scalar { value } => value,
        value => panic!("unexpected Temporal result {value:?}"),
    }
}

fn binding_ids(value: &veac_ir::TemporalProgramLibrary) -> Vec<String> {
    value
        .bindings
        .iter()
        .map(|item| item.id.to_string())
        .collect()
}

fn program_ids(value: &veac_ir::TemporalProgramLibrary) -> Vec<String> {
    value
        .programs
        .iter()
        .map(|item| item.id.to_string())
        .collect()
}

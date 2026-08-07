use veac_ir::{TemporalClock, TemporalClockOwner, TemporalValue};
use veac_lang::program::build_source;

const SOURCE: &str = r#"
fn media_card(key: identifier, source: Resource) -> Item {
  let value = item(key, item_enabled(), during(0s, 2s),
    source_media(source), source_timing_native());
  animate visual-opacity on clip(value) {
    clamp(source_time / 2s, 0.0, 1.0)
  }
}

fn main(context: Context) -> Project {
  let hero = image_resource(identifier("hero"), resource_file("assets/hero.png"),
    sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
  let content = media_card(identifier("content"), hero);
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(content);
  let timeline = sequence(identifier("main"), "素材时钟组件动画",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(visual);
  project(identifier("media-component"), project_settings(600))
    .with_resource(hero).with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn used_source_time_residualizes_one_typed_item_clock() {
    let built = build_source(SOURCE).unwrap();
    let envelope = built.envelope();
    let binding = &envelope.temporal.bindings[0];
    assert_eq!(binding.clocks.len(), 1);
    assert_eq!(binding.clocks[0].clock, TemporalClock::SourceTime);
    assert!(matches!(
        binding.clocks[0].owner,
        TemporalClockOwner::Item { .. }
    ));
    let program = &envelope.temporal.programs[0];
    let result = veac_ir::evaluate_temporal_program(
        program,
        &[veac_ir::TemporalEvaluationInput {
            input_id: binding.clocks[0].input_id,
            value: TemporalValue::Time {
                value: veac_ir::RationalTime::new(1, 1).unwrap(),
            },
        }],
        veac_ir::TemporalEvaluationLimits::default(),
    )
    .unwrap();
    assert_eq!(result, TemporalValue::Scalar { value: 0.5 });
}

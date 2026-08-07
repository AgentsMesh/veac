use veac_ir::{TemporalEvaluationInput, TemporalEvaluationLimits, TemporalValue};

pub const MAIN: &str = r#"
fn pulse(value: scalar) -> scalar { clamp(value * 2.0, 0.0, 1.0) }

animate visual-opacity on clip(@demo, @main, @visual, @first) {
  pulse(progress)
}

fn main(context: Context) -> Project {
  let first = item(identifier("first"), item_enabled(), during(0s, 2s),
    source_generated(generator_solid(#234567ff)), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let layer = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(first);
  let timeline = sequence(identifier("main"), "主时间线",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(layer);
  project(identifier("demo"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

pub const SINK_MATRIX: &str = r#"
fn main(context: Context) -> Project {
  let visual = item(identifier("visual-clip"), item_enabled(), during(0s, 3s),
    source_generated(generator_transparent()), source_timing_native())
    .with_visual(visual_style(
      visual_layout(placement_anchor(anchor_center(), vector(0.0, 0.0)), frame_none(),
        transform_2d(transform_motion(point_constant(point(0px, 0px)),
          vector_constant(vector(1.0, 1.0)), angle_constant(0deg)),
          transform_geometry(vector(0.0, 0.0), flip_none(), vector(0.5, 0.5),
            crop_animated(rect_constant(rect(0.0, 0.0, 1.0, 1.0)))))),
      visual_surface(percent_constant(100%), compositing(0, blend_normal()), card_none()),
      [], color_pipeline_none()));
  let audio = item(identifier("audio-clip"), item_enabled(), during(0s, 3s),
    source_generated(generator_silence()), source_timing_native())
    .with_audio(audio_style(scalar_constant(1.0), scalar_constant(0.0),
      audio_playback(false, false, pitch_preserve()), [], audio_crossfade_none()));
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visuals = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(visual);
  let sounds = audio_layer(identifier("audio"), 1, placement_free(), state,
    track_routing_default()).with_item(audio);
  let timeline = sequence(identifier("main"), "属性绑定矩阵",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(visuals).with_layer(sounds);
  project(identifier("matrix"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

pub const MEDIA: &str = r#"
fn main(context: Context) -> Project {
  let hero = image_resource(identifier("hero"), resource_file("assets/hero.png"),
    sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
  let other = image_resource(identifier("other"), resource_file("assets/other.png"),
    sha256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"));
  let clip = item(identifier("hero-clip"), item_enabled(), during(0s, 3s),
    source_media(hero), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let layer = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(clip);
  let timeline = sequence(identifier("main"), "素材时间动画",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(layer);
  project(identifier("media"), project_settings(600))
    .with_resource(hero).with_resource(other).with_sequence(timeline).entry(timeline)
}
"#;

pub fn evaluate(envelope: &veac_ir::ProjectEnvelope, value: TemporalValue) -> TemporalValue {
    let binding = &envelope.temporal.bindings[0];
    veac_ir::evaluate_temporal_program(
        &envelope.temporal.programs[0],
        &[TemporalEvaluationInput {
            input_id: binding.clocks[0].input_id,
            value,
        }],
        TemporalEvaluationLimits::default(),
    )
    .unwrap()
}

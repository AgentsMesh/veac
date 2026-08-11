use veac_ir::{Animatable, Effect};

use super::support;

const SOURCE: &str = r#"
animate effect-parameter on clip-effect(
  @directional-blur-v8, @main, @visual, @bound, @motion-bound, @angle_degrees
) { progress * 360.0 }
animate effect-parameter on clip-effect(
  @directional-blur-v8, @main, @visual, @bound, @motion-bound, @radius
) { progress * 100.0 }

fn shot(key: identifier, start: time, value: Effect) -> Item {
  item(key, item_enabled(), during(start, 1s),
    source_generated(generator_solid(#336699ff)), source_timing_native()
  ).with_effect(value)
}

fn main(context: Context) -> Project {
  let constant = shot(identifier("constant"), 0s, video_directional_blur_effect(
    identifier("motion-constant"), effect_enabled(effect_window_full()),
    angle_constant(45deg), length_constant(12px)
  ));
  let keyed = shot(identifier("keyed"), 1s, video_directional_blur_effect(
    identifier("motion-keyed"), effect_enabled(effect_window_full()),
    angle_keyframes([
      angle_keyframe(identifier("start"), 0s, 0deg, interpolation_ease_out()),
      angle_keyframe(identifier("end"), 1s, 180deg, interpolation_linear())
    ]),
    length_keyframes([
      length_keyframe(identifier("start"), 0s, 0px, interpolation_ease_out()),
      length_keyframe(identifier("end"), 1s, 48px, interpolation_linear())
    ])
  ));
  let bound = shot(identifier("bound"), 2s, video_directional_blur_effect(
    identifier("motion-bound"), effect_enabled(effect_window_full()),
    angle_constant(0deg), length_constant(0px)
  ));
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_items([constant, keyed, bound]);
  let timeline = sequence(identifier("main"), "方向性运动模糊",
    sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000)
  ).with_layer(visual);
  project(identifier("directional-blur-v8"), project_settings(600))
    .with_sequence(timeline).entry(timeline)
}
"#;

#[test]
fn directional_blur_lowers_constants_keyframes_and_bindings() {
    let envelope = support::envelope(SOURCE);
    let clips = &envelope.project.sequences[0].tracks[0].clips;
    assert_eq!(clips.len(), 3);
    let Effect::VideoDirectionalBlur {
        angle_degrees,
        radius,
    } = &clips[0].effects[0].effect
    else {
        panic!("expected directional blur")
    };
    assert_eq!(angle_degrees, &Animatable::constant(45.0));
    assert_eq!(radius, &Animatable::constant(12.0));

    let Effect::VideoDirectionalBlur {
        angle_degrees: Animatable::Keyframes { keyframes: angles },
        radius: Animatable::Keyframes { keyframes: radii },
    } = &clips[1].effects[0].effect
    else {
        panic!("expected directional blur keyframes")
    };
    assert_eq!(
        angles.iter().map(|key| key.value).collect::<Vec<_>>(),
        [0.0, 180.0]
    );
    assert_eq!(
        radii.iter().map(|key| key.value).collect::<Vec<_>>(),
        [0.0, 48.0]
    );

    let Effect::VideoDirectionalBlur {
        angle_degrees: Animatable::Binding { .. },
        radius: Animatable::Binding { .. },
    } = &clips[2].effects[0].effect
    else {
        panic!("expected directional blur bindings")
    };
    assert_eq!(envelope.temporal.bindings.len(), 2);
    veac_ir::validate(&envelope).unwrap();
}

#[test]
fn directional_blur_constructor_keeps_animation_types_closed() {
    let wrong_angle = SOURCE.replace("angle_constant(45deg)", "length_constant(45px)");
    assert!(veac_lang::program::build_source(&wrong_angle).is_err());
    let wrong_radius = SOURCE.replace("length_constant(12px)", "angle_constant(12deg)");
    assert!(veac_lang::program::build_source(&wrong_radius).is_err());
}

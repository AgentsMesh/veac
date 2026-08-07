use veac_ir::{Animatable, Length, LengthUnit, Point, Rect, Vec2};

use super::super::support;

const SOURCE: &str = r#"
fn styled(
  key: identifier, start: time,
  position: PointAnimation, scale: VectorAnimation, rotation: AngleAnimation,
  crop: RectAnimation, opacity: PercentAnimation, edge: LengthAnimation
) -> Item {
  item(
    key, item_enabled(), during(start, 2s),
    source_generated(generator_transparent()), source_timing_native()
  ).with_visual(visual_style(
    visual_layout(
      placement_anchor(anchor_center(), vector(0.0, 0.0)), frame_none(),
      transform_2d(
        transform_motion(position, scale, rotation),
        transform_geometry(vector(0.0, 0.0), flip_none(), vector(0.5, 0.5), crop_animated(crop))
      )
    ),
    visual_surface(opacity, compositing(0, blend_normal()), card_none()),
    [mask(
      mask_rectangle(),
      mask_motion(vector_constant(vector(0.5, 0.5)), vector_constant(vector(1.0, 1.0)), angle_constant(0deg)),
      mask_edge(edge, length_constant(0px)), false
    )],
    color_pipeline_none()
  ))
}

fn main(context: Context) -> Project {
  let constant = styled(
    identifier("constant"), 0s,
    point_constant(point(1px, 2px)), vector_constant(vector(1.0, 1.0)), angle_constant(5deg),
    rect_constant(rect(0.0, 0.0, 1.0, 1.0)), percent_constant(75%), length_constant(2px)
  );
  let keyed = styled(
    identifier("keyed"), 2s,
    point_keyframes([
      point_keyframe(identifier("start"), 0s, point(0px, 0px), interpolation_linear()),
      point_keyframe(identifier("end"), 1s, point(10px, 20px), interpolation_ease_out())
    ]),
    vector_keyframes([
      vector_keyframe(identifier("start"), 0s, vector(1.0, 1.0), interpolation_linear()),
      vector_keyframe(identifier("end"), 1s, vector(1.2, 0.8), interpolation_ease_in())
    ]),
    angle_keyframes([
      angle_keyframe(identifier("start"), 0s, 0deg, interpolation_hold()),
      angle_keyframe(identifier("end"), 1s, 30deg, interpolation_ease_in_out())
    ]),
    rect_keyframes([
      rect_keyframe(identifier("start"), 0s, rect(0.0, 0.0, 1.0, 1.0), interpolation_linear()),
      rect_keyframe(identifier("end"), 1s, rect(0.1, 0.2, 0.7, 0.6), interpolation_linear())
    ]),
    percent_keyframes([
      percent_keyframe(identifier("start"), 0s, 20%, interpolation_linear()),
      percent_keyframe(identifier("end"), 1s, 90%, interpolation_linear())
    ]),
    length_keyframes([
      length_keyframe(identifier("start"), 0s, 0px, interpolation_linear()),
      length_keyframe(identifier("end"), 1s, 8px, interpolation_linear())
    ])
  );
  let audio = item(
    identifier("audio"), item_enabled(), during(0s, 2s),
    source_generated(generator_silence()), source_timing_native()
  ).with_audio(audio_style(
    scalar_constant(0.8),
    scalar_keyframes([
      scalar_keyframe(identifier("start"), 0s, -0.5, interpolation_linear()),
      scalar_keyframe(identifier("end"), 1s, 0.5, interpolation_linear())
    ]),
    audio_playback(false, false, pitch_preserve()), [], audio_crossfade_none()
  ));
  let state = track_state(track_playback_enabled(), track_audio_audible(), track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state, track_routing_default())
    .with_item(constant).with_item(keyed);
  let audio_track = audio_layer(identifier("audio"), 1, placement_free(), state, track_routing_default())
    .with_item(audio);
  let sequence = sequence(identifier("main"), "动画值", sequence_settings(canvas(320px, 180px), frame_rate(30, 1), 48000))
    .with_layer(visual).with_layer(audio_track);
  project(identifier("animations"), project_settings(600)).with_sequence(sequence).entry(sequence)
}
"#;

#[test]
fn every_typed_animation_constant_and_keyframe_family_reaches_canonical_ir() {
    let value = support::envelope(SOURCE);
    let sequence = &value.project.sequences[0];
    let constant = sequence.tracks[0].clips[0].visual.as_ref().unwrap();
    assert_eq!(
        constant.transform.position,
        Animatable::constant(Point {
            x: Length {
                value: 1.0,
                unit: LengthUnit::Pixels
            },
            y: Length {
                value: 2.0,
                unit: LengthUnit::Pixels
            },
        })
    );
    assert_eq!(
        constant.transform.scale,
        Animatable::constant(Vec2 { x: 1.0, y: 1.0 })
    );
    assert_eq!(
        constant.transform.rotation_degrees,
        Animatable::constant(5.0)
    );
    assert_eq!(
        constant.transform.crop,
        Some(Animatable::constant(Rect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }))
    );
    assert_eq!(constant.opacity, Animatable::constant(0.75));
    assert_eq!(constant.masks[0].feather_pixels, Animatable::constant(2.0));

    let keyed = sequence.tracks[0].clips[1].visual.as_ref().unwrap();
    let curves = [
        keyed.transform.position.keyframes().unwrap().len(),
        keyed.transform.scale.keyframes().unwrap().len(),
        keyed.transform.rotation_degrees.keyframes().unwrap().len(),
        keyed
            .transform
            .crop
            .as_ref()
            .unwrap()
            .keyframes()
            .unwrap()
            .len(),
        keyed.opacity.keyframes().unwrap().len(),
        keyed.masks[0].feather_pixels.keyframes().unwrap().len(),
    ];
    assert_eq!(curves, [2; 6]);
    let audio = sequence.tracks[1].clips[0].audio.as_ref().unwrap();
    assert_eq!(audio.gain, Animatable::constant(0.8));
    assert_eq!(
        audio
            .pan
            .keyframes()
            .unwrap()
            .iter()
            .map(|key| key.value)
            .collect::<Vec<_>>(),
        [-0.5, 0.5]
    );
    assert_ne!(
        keyed.transform.position.keyframes().unwrap()[0].id,
        keyed.transform.scale.keyframes().unwrap()[0].id
    );
    assert!(veac_ir::validate(&value).is_ok());
}

use veac_lang::program::build_source;

use super::support::{item_duration, validate_canonical};

const SOURCE: &str = r#"fn twice(value: time) -> time { value * 2.0 }
fn timed(key: identifier, duration: time) -> Item {
  item(
    key, item_enabled(), during(0s, duration),
    source_generated(generator_transparent()), source_timing_native()
  )
}
fn main(context: Context) -> Project {
  let constant_duration = twice(250ms);
  let local_duration = twice(twice(1s));
  let state = track_state(
    track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked()
  );
  let layer = visual_layer(
    identifier("content"), 0, placement_free(), state, track_routing_default()
  )
    .with_item(timed(identifier("constant"), constant_duration))
    .with_item(timed(identifier("local"), local_duration))
    .with_item(timed(identifier("argument"), twice(4s)));
  let timeline = sequence(
    identifier("main"), "函数上下文",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)
  ).with_layer(layer);
  project(identifier("function-contexts"), project_settings(1000))
    .with_sequence(timeline).entry(timeline)
}"#;

#[test]
fn functions_execute_in_every_existing_expression_context() {
    let built = build_source(SOURCE).unwrap();
    assert_eq!(item_duration(&built, 0, 0, 0), "500ms");
    assert_eq!(item_duration(&built, 0, 0, 1), "4s");
    assert_eq!(item_duration(&built, 0, 0, 2), "8s");
    validate_canonical(&built);
}

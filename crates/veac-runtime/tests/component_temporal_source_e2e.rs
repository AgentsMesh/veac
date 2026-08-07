#[allow(unused_imports)]
#[path = "render_e2e/support/mod.rs"]
mod support;

use std::fs;

use tempfile::tempdir;
use veac_lang::program::build_path;

use support::*;

const FACTORY: &str = r#"module {
  export fn card(key: identifier, at: time, fill: color) -> Item {
    let value = item(key, item_enabled(), during(at, 1s),
      source_generated(generator_solid(fill)), source_timing_native());
    animate visual-opacity on clip(value) { private_fade(progress) }
  }

  fn private_fade(value: scalar) -> scalar { clamp(value, 0.0, 1.0) }
}"#;

const ENTRY: &str = r#"import "./factory.veac" as cards;

fn state() -> TrackState {
  track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked())
}

fn delivery_for(timeline: Sequence) -> Delivery {
  let picture = video_output(video_h264(), pixel_yuv420p(), alpha_opaque(),
    video_color_unspecified(), video_crf(18), gop_auto(), b_frames_auto(),
    video_profile_present(profile_h264_high()), video_level_auto());
  let artifact = deliverable_video(identifier("preview"), delivery_file("preview.mp4"),
    video_delivery(container_mp4(), picture, embedded_audio_none(), true,
      pass_single(), hardware_software()));
  delivery(identifier("preview"), timeline,
    raster_settings(canvas(96px, 54px), frame_rate(10, 1), caption_discard()), [artifact])
}

fn main(context: Context) -> Project {
  let red = cards.card(identifier("red"), 0s, #ed3b35ff);
  let green = cards.card(identifier("green"), 1s, #35c96bff);
  let layer = visual_layer(identifier("visual"), 0, placement_free(), state(),
    track_routing_default()).with_item(red).with_item(green);
  let timeline = sequence(identifier("main"), "组件动画 E2E",
    sequence_settings(canvas(96px, 54px), frame_rate(10, 1), 48000)).with_layer(layer);
  project(identifier("component-e2e"), project_settings(1000))
    .with_sequence(timeline).entry(timeline).with_delivery(delivery_for(timeline))
}"#;

#[test]
fn module_factory_instances_reach_shared_program_plan_and_distinct_pixels() {
    let directory = tempdir().unwrap();
    fs::write(directory.path().join("factory.veac"), FACTORY).unwrap();
    let entry = directory.path().join("main.veac");
    fs::write(&entry, ENTRY).unwrap();
    let envelope = build_path(&entry).unwrap().envelope().clone();
    veac_ir::validate(&envelope).unwrap();
    assert_temporal_instances(&envelope.temporal);

    let output = directory.path().join("component-temporal.mp4");
    let rendered = render(envelope, &BTreeMap::new(), &output);
    assert_temporal_instances(&rendered.plan.temporal);
    assert_eq!(rendered.plan.sequences[0].tracks[0].clips.len(), 2);

    let red_early = rgb_at(&output, 0.1, WIDTH / 2, HEIGHT / 2);
    let red_late = rgb_at(&output, 0.8, WIDTH / 2, HEIGHT / 2);
    let green_early = rgb_at(&output, 1.1, WIDTH / 2, HEIGHT / 2);
    let green_late = rgb_at(&output, 1.8, WIDTH / 2, HEIGHT / 2);
    assert!(
        red_late[0] > red_early[0] + 80,
        "{red_early:?}..{red_late:?}"
    );
    assert!(
        green_late[1] > green_early[1] + 70,
        "{green_early:?}..{green_late:?}"
    );
}

fn assert_temporal_instances(library: &TemporalProgramLibrary) {
    assert_eq!(library.programs.len(), 1);
    assert_eq!(library.bindings.len(), 2);
    assert_eq!(library.provenance.len(), 2);
    assert_ne!(library.bindings[0].id, library.bindings[1].id);
    assert_eq!(
        library.bindings[0].program_id,
        library.bindings[1].program_id
    );
}

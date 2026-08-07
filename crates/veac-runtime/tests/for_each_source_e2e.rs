#[allow(unused_imports)]
#[path = "render_e2e/support/mod.rs"]
mod support;

use tempfile::tempdir;
use veac_lang::program::build_source;

use support::*;

const SOURCE: &str = r#"
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
  let seed = item(identifier("seed"), item_enabled(), during(0s, 1s),
    source_generated(generator_solid(#123a72ff)), source_timing_native());
  let red_values = for key in [identifier("loop-red")] {
    item(key, item_enabled(), during(0s, 0.5s),
      source_generated(generator_solid(#ed3b35ff)), source_timing_native())
  };
  let red = fold(red_values, seed,
    fn(current: Item, next: Item) -> Item effect pure { next });
  let green_values = for key in [identifier("loop-green")] {
    item(key, item_enabled(), during(0.5s, 0.5s),
      source_generated(generator_solid(#35c96bff)), source_timing_native())
  };
  let green = fold(green_values, red,
    fn(current: Item, next: Item) -> Item effect pure { next });
  let layer = visual_layer(identifier("visual"), 0, placement_free(), state(),
    track_routing_default()).with_item(seed).with_item(red).with_item(green);
  let timeline = sequence(identifier("main"), "ForEach source E2E",
    sequence_settings(canvas(96px, 54px), frame_rate(10, 1), 48000)).with_layer(layer);
  project(identifier("for-each-e2e"), project_settings(1000))
    .with_sequence(timeline).entry(timeline).with_delivery(delivery_for(timeline))
}
"#;

#[test]
fn authored_for_each_reaches_stable_plan_entities_and_rendered_pixels() {
    let first = build_source(SOURCE).unwrap().envelope().clone();
    let second = build_source(SOURCE).unwrap().envelope().clone();
    let first_clips = &first.project.sequences[0].tracks[0].clips;
    let second_clips = &second.project.sequences[0].tracks[0].clips;
    assert_eq!(first_clips.len(), 3);
    assert_eq!(
        first_clips.iter().map(|clip| &clip.id).collect::<Vec<_>>(),
        second_clips.iter().map(|clip| &clip.id).collect::<Vec<_>>()
    );
    assert_iteration(first_clips, "loop-red");
    assert_iteration(first_clips, "loop-green");
    let generated_ids = [first_clips[1].id.clone(), first_clips[2].id.clone()];

    let temp = tempdir().unwrap();
    let output = temp.path().join("for-each-source.mp4");
    let rendered = render(first, &BTreeMap::new(), &output);
    let planned = &rendered.plan.sequences[0].tracks[0].clips;
    assert_eq!(planned.len(), 3);
    assert_eq!(planned[1].id.as_str(), generated_ids[0].as_str());
    assert_eq!(planned[2].id.as_str(), generated_ids[1].as_str());
    let early = rgb_at(&output, 0.25, WIDTH / 2, HEIGHT / 2);
    let late = rgb_at(&output, 0.75, WIDTH / 2, HEIGHT / 2);
    assert!(
        early[0] > early[1] + 80 && early[0] > early[2] + 80,
        "{early:?}"
    );
    assert!(late[1] > late[0] + 50 && late[1] > late[2] + 30, "{late:?}");
}

fn assert_iteration(clips: &[Clip], key: &str) {
    let clip = clips
        .iter()
        .find(|clip| {
            clip.authorship
                .as_ref()
                .and_then(|value| value.logical_path.last())
                .is_some_and(|value| value.as_str() == key)
        })
        .unwrap();
    let loops = &clip.authorship.as_ref().unwrap().events[0].iterations;
    assert_eq!(loops.len(), 1);
    assert_eq!(loops[0].index, 0);
}

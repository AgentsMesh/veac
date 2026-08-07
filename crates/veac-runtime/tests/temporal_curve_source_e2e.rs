#[allow(unused_imports)]
#[path = "render_e2e/support/mod.rs"]
mod support;

use tempfile::tempdir;
use veac_ir::{Animatable, TemporalNodeKind};
use veac_lang::program::build_source;
use veac_lang::program::expression::CORE_VERSION;

use support::*;

const SOURCE: &str = r#"
animate visual-opacity on clip(@curve-pixel, @main, @foreground, @marker) {
  sample_curve_linear(progress, [(0.0, 0.0), (1.0, 1.0)])
}

fn state() -> TrackState {
  track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked())
}

fn video(timeline: Sequence) -> Delivery {
  let picture = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(18), gop_auto(), b_frames_auto(),
    video_profile_present(profile_h264_high()), video_level_auto()
  );
  let artifact = deliverable_video(
    identifier("preview"), delivery_file("preview.mp4"),
    video_delivery(container_mp4(), picture, embedded_audio_none(), true,
      pass_single(), hardware_software())
  );
  delivery(identifier("preview"), timeline,
    raster_settings(canvas(96px, 54px), frame_rate(10, 1), caption_discard()),
    [artifact])
}

fn solid(key: identifier, fill: color) -> Item {
  item(key, item_enabled(), during(0s, 1s),
    source_generated(generator_solid(fill)), source_timing_native())
}

fn main(context: Context) -> Project {
  let background = visual_layer(identifier("background"), 0, placement_free(),
    state(), track_routing_default())
    .with_item(solid(identifier("field"), #123a72ff));
  let foreground = visual_layer(identifier("foreground"), 1, placement_free(),
    state(), track_routing_default())
    .with_item(solid(identifier("marker"), #ed3b35ff));
  let timeline = sequence(identifier("main"), "曲线像素测试",
    sequence_settings(canvas(96px, 54px), frame_rate(10, 1), 48000))
    .with_layer(background).with_layer(foreground);
  project(identifier("curve-pixel"), project_settings(1000))
    .with_sequence(timeline).entry(timeline).with_delivery(video(timeline))
}
"#;

#[test]
fn authored_curve_reaches_ffmpeg_and_changes_pixels() {
    let built = build_source(SOURCE).expect("source must compile through verified Core");
    assert_eq!(built.entry_function().body().version(), CORE_VERSION);
    let envelope = built.envelope();
    veac_ir::validate(envelope).expect("executed source must publish canonical IR");
    let clip = &envelope.project.sequences[0].tracks[1].clips[0];
    assert!(matches!(
        clip.visual.as_ref().unwrap().opacity,
        Animatable::Binding { .. }
    ));
    assert_curve_sample(&envelope.temporal);

    let temp = tempdir().unwrap();
    let output = temp.path().join("authored-curve.mp4");
    let rendered = render(envelope.clone(), &BTreeMap::new(), &output);
    assert_curve_sample(&rendered.plan.temporal);
    let graph = rendered.command.filter_graph.as_deref().unwrap();
    assert!(graph.contains("a='alpha(X\\,Y)*(if(lte("), "graph={graph}");
    assert_media_contract(&output, 0, 1.0);

    let early = rgb_at(&output, 0.1, WIDTH / 2, HEIGHT / 2);
    let late = rgb_at(&output, 0.8, WIDTH / 2, HEIGHT / 2);
    assert!(early[2] > early[0] + 35, "early pixel={early:?}");
    assert!(late[0] > late[2] + 70, "late pixel={late:?}");
    assert!(late[0] > early[0] + 100, "pixels={early:?}..{late:?}");
}

fn assert_curve_sample(library: &TemporalProgramLibrary) {
    assert_eq!(library.programs.len(), 1);
    let samples = library.programs[0]
        .nodes
        .iter()
        .filter_map(|node| match &node.kind {
            TemporalNodeKind::CurveSample { keys, .. } => Some(keys),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(samples.len(), 1);
    assert_eq!(samples[0].len(), 2);
    assert!(samples[0]
        .iter()
        .all(|key| key.interpolation == Interpolation::Linear));
}

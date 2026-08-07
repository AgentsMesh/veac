#[allow(unused_imports)]
#[path = "render_e2e/support/mod.rs"]
mod support;

use tempfile::tempdir;
use veac_ir::{
    plugin_effect, Animatable, Effect, PluginBackend, REFERENCE_MONOCHROME_DIGEST,
    REFERENCE_MONOCHROME_EFFECT_TYPE,
};
use veac_lang::program::build_source;
use veac_plan::ResolvedRenderPlan;

use support::*;

const SOURCE: &str = r#"
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

fn main(context: Context) -> Project {
  let primary_item = item(identifier("card"), item_enabled(), during(0s, 1s),
    source_generated(generator_solid(#e83a78ff)), source_timing_native())
    .with_effect(video_plugin_scalar_effect(
      identifier("mono"), effect_enabled(effect_window_full()),
      plugin_reference_monochrome_v1(), scalar_constant(__AMOUNT__)
    ));
  let layer = visual_layer(identifier("visual"), 0, placement_free(),
    state(), track_routing_default()).with_item(primary_item);
  let timeline = sequence(identifier("main"), "插件像素测试",
    sequence_settings(canvas(96px, 54px), frame_rate(10, 1), 48000))
    .with_layer(layer);
  project(identifier("plugin-pixel"), project_settings(1000))
    .with_sequence(timeline).entry(timeline).with_delivery(video(timeline))
}
"#;

#[test]
fn authored_content_addressed_plugin_reaches_ffmpeg_pixels() {
    let temp = tempdir().unwrap();
    let color = build("0.0");
    let mono = build("1.0");
    assert_descriptor(&color);
    assert_descriptor(&mono);

    let color_path = temp.path().join("color.mp4");
    let mono_path = temp.path().join("mono.mp4");
    let color_render = render(color, &BTreeMap::new(), &color_path);
    let mono_render = render(mono, &BTreeMap::new(), &mono_path);
    assert_adapter(&color_render.command, "0");
    assert_adapter(&mono_render.command, "1");
    assert_exact_schema(&mono_render.plan);

    let colored = rgb_at(&color_path, 0.5, WIDTH / 2, HEIGHT / 2);
    let grayscale = rgb_at(&mono_path, 0.5, WIDTH / 2, HEIGHT / 2);
    assert!(chroma(colored) > 80, "colored={colored:?}");
    assert!(chroma(grayscale) < 8, "grayscale={grayscale:?}");
    assert!(changed_channels(&colored, &grayscale) >= 2);
}

fn build(amount: &str) -> ProjectEnvelope {
    let built = build_source(&SOURCE.replace("__AMOUNT__", amount)).unwrap();
    let envelope = built.envelope().clone();
    veac_ir::validate(&envelope).unwrap();
    envelope
}

fn assert_descriptor(envelope: &ProjectEnvelope) {
    let effect = &envelope.project.sequences[0].tracks[0].clips[0].effects[0];
    let Effect::VideoPluginReferenceMonochromeV1 {
        descriptor_digest,
        amount,
    } = &effect.effect
    else {
        panic!("typed plugin effect")
    };
    assert_eq!(effect.kind().type_name(), REFERENCE_MONOCHROME_EFFECT_TYPE);
    assert_eq!(descriptor_digest.as_str(), REFERENCE_MONOCHROME_DIGEST);
    assert!(matches!(amount, Animatable::Constant { .. }));
    let descriptor = plugin_effect(effect.kind()).unwrap();
    assert_eq!(descriptor.digest, REFERENCE_MONOCHROME_DIGEST);
    assert!(descriptor.digest_matches());
    assert!(descriptor.supports(PluginBackend::Ffmpeg8));
    assert_eq!(
        descriptor.descriptor_constructor,
        "plugin_reference_monochrome_v1"
    );
    assert_eq!(
        descriptor.application_constructor,
        "video_plugin_scalar_effect"
    );
}

fn assert_adapter(command: &veac_codegen::emitter::BackendCommand, amount: &str) {
    let graph = command.filter_graph.as_deref().unwrap();
    assert!(graph.contains(&format!("hue=s='1-({amount})'")), "{graph}");
}

fn assert_exact_schema(plan: &ResolvedRenderPlan) {
    let effect = &plan.sequences[0].tracks[0].clips[0].effects[0];
    let mut missing = serde_json::to_value(effect).unwrap();
    missing["effect"].as_object_mut().unwrap().remove("amount");
    assert!(serde_json::from_value::<veac_plan::ResolvedEffect>(missing).is_err());

    let mut extra = serde_json::to_value(effect).unwrap();
    extra["effect"]["backend_flag"] = serde_json::json!(true);
    assert!(serde_json::from_value::<veac_plan::ResolvedEffect>(extra).is_err());
}

fn chroma(pixel: [u8; 3]) -> u8 {
    pixel.iter().max().unwrap() - pixel.iter().min().unwrap()
}

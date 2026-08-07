pub(crate) const EXECUTABLE_SOURCE: &str = r#"fn main(context: Context) -> Project {
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let scene_item = item(
    identifier("background"), item_enabled(), during(0s, 200ms),
    source_generated(generator_solid(#112233ff)), source_timing_native());
  let layer = visual_layer(
    identifier("visual"), 0, placement_free(), state, track_routing_default())
    .with_item(scene_item);
  let timeline = sequence(
    identifier("main"), "CLI 可执行测试时间线",
    sequence_settings(canvas(32px, 24px), frame_rate(10, 1), 48000))
    .with_layer(layer);
  let video_spec = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(23), gop_auto(), b_frames_auto(), video_profile_auto(), video_level_auto());
  let artifact = deliverable_video(
    identifier("main"), delivery_file("render.mp4"),
    video_delivery(container_mp4(), video_spec, embedded_audio_none(),
      true, pass_single(), hardware_software()));
  let delivery_spec = delivery(
    identifier("main"), timeline,
    raster_settings(canvas(32px, 24px), frame_rate(10, 1), caption_discard()),
    [artifact]);
  project(identifier("cli-e2e"), project_settings(1000))
    .with_sequence(timeline).entry(timeline).with_delivery(delivery_spec)
}
"#;

pub(crate) const MEDIA_SOURCE: &str = r#"fn main(context: Context) -> Project {
  let footage = video_resource(
    identifier("footage"), resource_file("clip.mp4"),
    sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
    stream_intent(stream_auto(), stream_disabled()));
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let scene_item = item(
    identifier("footage-1"), item_enabled(), during(0s, 200ms),
    source_media(footage), source_timing_native());
  let layer = video_layer(
    identifier("base"), 0, placement_free(), state, track_routing_default())
    .with_item(scene_item);
  let timeline = sequence(
    identifier("main"), "CLI 媒体测试时间线",
    sequence_settings(canvas(32px, 24px), frame_rate(10, 1), 48000))
    .with_layer(layer);
  let video_spec = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(23), gop_auto(), b_frames_auto(), video_profile_auto(), video_level_auto());
  let artifact = deliverable_video(
    identifier("main"), delivery_file("media-render.mp4"),
    video_delivery(container_mp4(), video_spec, embedded_audio_none(),
      true, pass_single(), hardware_software()));
  let delivery_spec = delivery(
    identifier("main"), timeline,
    raster_settings(canvas(32px, 24px), frame_rate(10, 1), caption_discard()),
    [artifact]);
  project(identifier("media-e2e"), project_settings(600))
    .with_resource(footage).with_sequence(timeline).entry(timeline).with_delivery(delivery_spec)
}
"#;

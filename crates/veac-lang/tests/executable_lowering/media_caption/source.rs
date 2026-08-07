pub(super) const SOURCE: &str = r#"
fn style(font: Resource) -> TextStyle {
  let fonts = font_stack(font_resource_ref(font), []);
  let metrics = text_metrics(
    fonts, weight_normal(), font_style_normal(), 32px, 0px, 1.0, #fef3c7ff);
  let layout = text_layout(
    text_box_auto(), text_wrap_none(), text_overflow_visible(),
    text_align_center(), text_align_middle(), writing_horizontal_tb(), orientation_mixed());
  text_style(metrics, layout, text_path_none(),
    text_decoration(text_background_none(), text_outline_none(), shadow_none()),
    [], text_animation_none())
}

fn main(context: Context) -> Project {
  let image = image_resource(identifier("poster"), resource_file("assets/poster.png"),
    sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"));
  let audio = audio_resource(identifier("voice"), resource_file("assets/voice.wav"),
    sha256("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"),
    stream_auto());
  let font = font_resource(identifier("caption-font"), resource_file("assets/caption.ttf"),
    sha256("cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc"));
  let poster = item(identifier("poster"), item_enabled(), during(0s, 3s),
    source_media(image), source_timing_native());
  let voice = item(identifier("voice"), item_enabled(), during(0s, 3s),
    source_media(audio), source_timing_native());
  let opening = item(identifier("opening"), item_enabled(), during(0s, 1s),
    source_caption("开场字幕", style(font)), source_timing_native());
  let quote = item(identifier("quote"), item_enabled(), during(1s, 2s),
    source_caption_speaker("开始创作", "旁白", style(font)), source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let visual = visual_layer(identifier("visual"), 0, placement_free(), state,
    track_routing_default()).with_item(poster);
  let sound = audio_layer(identifier("audio"), 1, placement_free(), state,
    track_routing_default()).with_item(voice);
  let captions = caption_layer(identifier("captions"), 2, placement_free(), state,
    track_routing_default()).with_item(opening).with_item(quote);
  let timeline = sequence(identifier("main"), "媒体与字幕",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
    .with_layer(visual).with_layer(sound).with_layer(captions);
  let video = video_output(
    video_h264(), pixel_yuv420p(), alpha_opaque(), video_color_unspecified(),
    video_crf(23), gop_auto(), b_frames_auto(), video_profile_auto(), video_level_auto());
  let movie = deliverable_video(identifier("movie"), delivery_file("preview.mp4"),
    video_delivery(container_mp4(), video,
      embedded_audio_present(audio_output(audio_aac(), 48000, 2)),
      false, pass_single(), hardware_software()));
  let output = delivery(identifier("preview"), timeline,
    raster_settings(canvas(640px, 360px), frame_rate(30, 1), caption_burn_in()), [movie]);
  project(identifier("media-caption"), project_settings(600))
    .with_resource(image).with_resource(audio).with_resource(font)
    .with_sequence(timeline).entry(timeline).with_delivery(output)
}
"#;

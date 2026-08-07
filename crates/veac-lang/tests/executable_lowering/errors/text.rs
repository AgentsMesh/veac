use super::assert_lower_reason;

#[test]
fn invalid_text_content_and_style_are_rejected_during_lowering() {
    for (content, size) in [("", "36px"), ("title", "0px")] {
        let source = format!(r#"source_text("{content}", style(font))"#);
        assert_lower_reason(
            &project(&source, size, "visual_layer"),
            "EXECUTABLE_LOWER_TEXT",
        );
    }
}

#[test]
fn invalid_caption_speakers_are_rejected_during_lowering() {
    for speaker in [" ".to_owned(), "x".repeat(257), "line\\nbreak".to_owned()] {
        let source = format!(r#"source_caption_speaker("text", "{speaker}", style(font))"#);
        assert_lower_reason(
            &project(&source, "36px", "caption_layer"),
            "EXECUTABLE_LOWER_TEXT",
        );
    }
}

fn project(source: &str, size: &str, layer: &str) -> String {
    format!(
        r#"
fn style(font: Resource) -> TextStyle {{
  let fonts = font_stack(font_resource_ref(font), []);
  let metrics = text_metrics(fonts, weight_normal(), font_style_normal(),
    {size}, 0px, 1.0, #ffffffff);
  let layout = text_layout(text_box_auto(), text_wrap_none(), text_overflow_visible(),
    text_align_center(), text_align_middle(), writing_horizontal_tb(), orientation_mixed());
  text_style(metrics, layout, text_path_none(),
    text_decoration(text_background_none(), text_outline_none(), shadow_none()),
    [], text_animation_none())
}}
fn main(context: Context) -> Project {{
  let font = font_resource(identifier("font"), resource_file("assets/font.ttf"),
    sha256("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"));
  let clip = item(identifier("text"), item_enabled(), during(0s, 1s),
    {source}, source_timing_native());
  let state = track_state(track_playback_enabled(), track_audio_audible(),
    track_isolation_normal(), track_editing_unlocked());
  let layer = {layer}(identifier("text"), 0, placement_free(), state,
    track_routing_default()).with_item(clip);
  let timeline = sequence(identifier("main"), "错误文本",
    sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000)).with_layer(layer);
  project(identifier("demo"), project_settings(600)).with_resource(font)
    .with_sequence(timeline).entry(timeline)
}}
"#
    )
}

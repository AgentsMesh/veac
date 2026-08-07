use super::support;

const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

#[test]
fn non_visual_transition_endpoints_fail_before_canonical_publication() {
    for source in [audio_project(), caption_project()] {
        let error = support::error(&source);
        assert_eq!(error.code, "PROGRAM_EXECUTABLE_IR");
        assert!(error.message.contains("EXECUTABLE_LOWER_IR_VALIDATION"));
    }
}

fn audio_project() -> String {
    project(
        &format!(
            r#"let asset = audio_resource(identifier("asset"), resource_file("audio.wav"),
              sha256("{DIGEST}"), stream_auto());
            let first = item(identifier("first"), item_enabled(), during(0s, 2s),
              source_media(asset), source_timing_native());
            let second = item(identifier("second"), item_enabled(), during(1s, 2s),
              source_media(asset), source_timing_native());"#
        ),
        "audio_layer(identifier(\"layer\"), 0, placement_free(), state, \
         track_routing_default()).with_item(first).with_item(second)",
    )
}

fn caption_project() -> String {
    project(
        &format!(
            r#"let asset = font_resource(identifier("asset"), resource_file("font.ttf"),
              sha256("{DIGEST}"));
            let fonts = font_stack(font_resource_ref(asset), []);
            let metrics = text_metrics(fonts, weight_normal(), font_style_normal(),
              32px, 0px, 1.0, #ffffffff);
            let layout = text_layout(text_box_auto(), text_wrap_none(), text_overflow_visible(),
              text_align_center(), text_align_middle(), writing_horizontal_tb(), orientation_mixed());
            let style = text_style(metrics, layout, text_path_none(),
              text_decoration(text_background_none(), text_outline_none(), shadow_none()),
              [], text_animation_none());
            let first = item(identifier("first"), item_enabled(), during(0s, 2s),
              source_caption("one", style), source_timing_native());
            let second = item(identifier("second"), item_enabled(), during(1s, 2s),
              source_caption("two", style), source_timing_native());"#
        ),
        "caption_layer(identifier(\"layer\"), 0, placement_free(), state, \
         track_routing_default()).with_item(first).with_item(second)",
    )
}

fn project(definitions: &str, layer: &str) -> String {
    format!(
        r#"fn main(context: Context) -> Project {{
          {definitions}
          let relation = relation_transition(
            identifier("bad"), first, second, transition_dissolve(1s));
          let state = track_state(track_playback_enabled(), track_audio_audible(),
            track_isolation_normal(), track_editing_unlocked());
          let timeline = sequence(identifier("main"), "错误关系",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
            .with_layer({layer}).with_relation(relation);
          project(identifier("demo"), project_settings(600)).with_resource(asset)
            .with_sequence(timeline).entry(timeline)
        }}"#
    )
}

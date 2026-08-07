use super::support;

#[path = "errors/text.rs"]
mod text;

#[test]
fn attached_visual_source_on_audio_layer_is_rejected_by_runtime_affinity() {
    let source = r#"
fn main(context: Context) -> Project {
    let bad = item(identifier("bad"), item_enabled(), during(0s, 1s),
        source_generated(generator_transparent()), source_timing_native());
    let state = track_state(track_playback_enabled(), track_audio_audible(),
        track_isolation_normal(), track_editing_unlocked());
    let audio = audio_layer(identifier("audio"), 0, placement_free(), state,
        track_routing_default()).with_item(bad);
    let timeline = sequence(identifier("main"), "错误轨道",
        sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
        .with_layer(audio);
    project(identifier("demo"), project_settings(600))
        .with_sequence(timeline).entry(timeline)
}
"#;
    let error = support::error(source);
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_IR");
    assert!(error.message.contains("EXECUTABLE_LOWER_IR_VALIDATION"));
}

#[test]
fn invalid_time_ranges_and_unsafe_ticks_are_rejected() {
    for (start, duration) in [("-1s", "1s"), ("0s", "0s"), ("0s", "9007199254740992s")] {
        let item = support::solid("bad", "#ffffffff", start, duration);
        assert_lower_reason(&support::visual_project(&[&item]), "EXECUTABLE_LOWER_TIME");
    }
}

#[test]
fn unbounded_time_precision_is_rejected_before_ir_publication() {
    let item = support::solid("bad", "#ffffffff", "0s", "0.0000000001s");
    assert_lower_reason(&support::visual_project(&[&item]), "EXECUTABLE_LOWER_TIME");
}

#[test]
fn invalid_canvas_and_frame_rate_have_stable_lowering_diagnostics() {
    for (canvas, rate, reason) in [
        ("0px, 360px", "30", "EXECUTABLE_LOWER_GRAPH"),
        ("640px, 360px", "0", "EXECUTABLE_LOWER_GRAPH"),
    ] {
        let source = format!(
            r#"fn main(context: Context) -> Project {{
                let timeline = sequence(identifier("main"), "错误设置",
                    sequence_settings(canvas({canvas}), frame_rate({rate}, 1), 48000));
                project(identifier("demo"), project_settings(600))
                    .with_sequence(timeline).entry(timeline)
            }}"#
        );
        assert_lower_reason(&source, reason);
    }
}

#[test]
fn canonical_ir_validation_failures_have_a_distinct_public_code() {
    let source = r#"fn main(context: Context) -> Project {
        let timeline = sequence(identifier("main"), "超限画布",
            sequence_settings(canvas(100000px, 360px), frame_rate(30, 1), 48000));
        project(identifier("demo"), project_settings(600))
            .with_sequence(timeline).entry(timeline)
    }"#;
    let error = support::error(source);
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_IR");
    assert!(error.message.contains("EXECUTABLE_LOWER_IR_VALIDATION"));
}

pub(super) fn assert_lower_reason(source: &str, reason: &str) {
    let error = support::error(source);
    assert_eq!(error.code, "PROGRAM_EXECUTABLE_LOWER");
    assert!(error.message.contains(reason), "{}", error.message);
}

use veac_ir::{
    FrameSynthesisPolicy, PlaybackDirection, SourceOutOfRangePolicy, SourceTimeInterpolation,
    SourceTimeMap, StreamChoice,
};

use super::super::support;

#[test]
fn mapped_source_time_covers_every_closed_policy_and_curve_shape() {
    let cases = [
        ("frame_nearest()", "out_of_range_strict()"),
        ("frame_blend()", "out_of_range_hold_first()"),
        ("frame_motion_compensated()", "out_of_range_hold_last()"),
        ("frame_nearest()", "out_of_range_hold_both()"),
    ];
    for (frame, boundary) in cases {
        let envelope = support::envelope(&mapped_project(&format!(
            "source_time_linear(2s, 0.5, 3, playback_reverse()), {frame}, {boundary}"
        )));
        let mapping = support::clip(&envelope, 0).source_mapping.as_ref().unwrap();
        let SourceTimeMap::Linear {
            direction, repeat, ..
        } = mapping.time_map
        else {
            panic!("expected linear source map")
        };
        assert_eq!(direction, PlaybackDirection::Reverse);
        assert_eq!(repeat, 3);
    }
    let curve = support::envelope(&mapped_project(
        "source_time_curve([source_time_segment(1s, 0s, 1s, segment_linear()), \
         source_time_segment(1s, 1s, 1s, segment_hold())]), frame_blend(), \
         out_of_range_hold_both()",
    ));
    let mapping = support::clip(&curve, 0).source_mapping.as_ref().unwrap();
    assert_eq!(mapping.frame_synthesis, FrameSynthesisPolicy::Blend);
    assert_eq!(mapping.out_of_range, SourceOutOfRangePolicy::HoldBoth);
    let SourceTimeMap::Curve { segments } = &mapping.time_map else {
        panic!("expected curve source map")
    };
    assert_eq!(segments[0].interpolation, SourceTimeInterpolation::Linear);
    assert_eq!(segments[1].interpolation, SourceTimeInterpolation::Hold);
}

#[test]
fn video_stream_global_and_disabled_choices_lower_exactly() {
    let source = mapped_project_with_stream(
        "source_time_linear(0s, 1.0, 1, playback_forward()), frame_nearest(), \
         out_of_range_strict()",
        "stream_intent(stream_global(7), stream_disabled())",
    );
    let envelope = support::envelope(&source);
    assert_eq!(
        envelope.project.materials[0].stream_intent.video,
        StreamChoice::GlobalIndex { global_index: 7 }
    );
    assert_eq!(
        envelope.project.materials[0].stream_intent.audio,
        StreamChoice::Disabled
    );
}

#[test]
fn remote_resources_and_freeze_intrinsic_time_lower() {
    let mapping = "source_time_linear(0s, 1.0, 1, playback_forward()), \
        frame_nearest(), out_of_range_strict()";
    let source = mapped_project(mapping)
        .replace(
            "resource_file(\"assets/media.mp4\")",
            "resource_remote_http(\"https://cdn.example.test/media.mp4\")",
        )
        .replace(
            "source_media(media), source_timing_mapped(source_mapping(source_time_linear(0s, 1.0, 1, playback_forward()), frame_nearest(), out_of_range_strict()))",
            "source_freeze_frame(media, 500ms), source_timing_native()",
        );
    let envelope = support::envelope(&source);
    assert!(matches!(
        envelope.project.materials[0].source,
        veac_ir::MaterialSource::Remote { .. }
    ));
    assert!(matches!(
        support::clip(&envelope, 0).source,
        veac_ir::ClipSource::FreezeFrame { .. }
    ));
}

#[test]
fn mapped_time_and_stream_integer_overflow_failures_are_lowering_errors() {
    for mapping in [
        "source_time_linear(0s, 1.0, -1, playback_forward()), frame_nearest(), out_of_range_strict()",
        "source_time_linear(0s, 1.0, 4294967296, playback_forward()), frame_nearest(), out_of_range_strict()",
    ] {
        assert_eq!(
            support::error(&mapped_project(mapping)).code,
            "PROGRAM_EXECUTABLE_LOWER"
        );
    }
    let source = mapped_project_with_stream(
        "source_time_linear(0s, 1.0, 1, playback_forward()), frame_nearest(), out_of_range_strict()",
        "stream_intent(stream_global(-1), stream_disabled())",
    );
    assert_eq!(support::error(&source).code, "PROGRAM_EXECUTABLE_LOWER");
    for at in ["9223372036854775808s", "0.0000000001s"] {
        let source = mapped_project(
            "source_time_linear(0s, 1.0, 1, playback_forward()), frame_nearest(), out_of_range_strict()",
        );
        let source = source.replace(
            "source_media(media), source_timing_mapped(source_mapping(source_time_linear(0s, 1.0, 1, playback_forward()), frame_nearest(), out_of_range_strict()))",
            &format!("source_freeze_frame(media, {at}), source_timing_native()"),
        );
        assert_eq!(support::error(&source).code, "PROGRAM_EXECUTABLE_LOWER");
    }
}

fn mapped_project(mapping: &str) -> String {
    mapped_project_with_stream(mapping, "stream_intent(stream_auto(), stream_disabled())")
}

fn mapped_project_with_stream(mapping: &str, streams: &str) -> String {
    format!(
        r#"fn main(context: Context) -> Project {{
          let media = video_resource(identifier("media"), resource_file("assets/media.mp4"),
            sha256("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"),
            {streams});
          let clip = item(identifier("clip"), item_enabled(), during(0s, 2s),
            source_media(media), source_timing_mapped(source_mapping({mapping})));
          let state = track_state(track_playback_enabled(), track_audio_audible(),
            track_isolation_normal(), track_editing_unlocked());
          let layer = video_layer(identifier("video"), 0, placement_magnetic(), state,
            track_routing_default()).with_item(clip);
          let timeline = sequence(identifier("main"), "源时间覆盖",
            sequence_settings(canvas(640px, 360px), frame_rate(30, 1), 48000))
            .with_layer(layer);
          project(identifier("coverage"), project_settings(600)).with_resource(media)
            .with_sequence(timeline).entry(timeline)
        }}"#
    )
}

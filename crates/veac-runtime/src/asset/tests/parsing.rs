use std::error::Error;

use veac_ir::{HashAlgorithm, ProbedStreamType, Rational, RationalTime, VideoCadence};

use super::*;

#[test]
fn normalizes_complete_ffprobe_output_without_floating_point_time() {
    let snapshot = parse(&complete_output(), auto_stream_intent()).unwrap();
    assert_eq!(snapshot.schema_version, PROBE_SCHEMA_VERSION);
    assert_eq!(snapshot.engine, FIXTURE_PROBE_ENGINE);
    assert_eq!(snapshot.selection_policy, STREAM_SELECTION_POLICY);
    assert_eq!(snapshot.container_format, "mov,mp4,m4a,3gp,3g2,mj2");
    assert_eq!(snapshot.container_brand.as_deref(), Some("isom"));
    assert_eq!(snapshot.observed_identity, identity());
    assert_eq!(
        snapshot.container_duration,
        Some(RationalTime::new(25, 2).unwrap())
    );
    assert_eq!(
        snapshot
            .streams
            .iter()
            .map(|stream| (stream.global_index, stream.type_index, stream.media_type))
            .collect::<Vec<_>>(),
        vec![
            (2, 0, ProbedStreamType::Video),
            (4, 0, ProbedStreamType::Audio),
            (6, 0, ProbedStreamType::Subtitle),
        ]
    );

    let video = selected_video(&snapshot).unwrap();
    assert_eq!(video.codec, "h264");
    assert_eq!(video.start_time, Some(RationalTime::new(1, 4).unwrap()));
    assert_eq!(video.duration, Some(RationalTime::new(12, 1).unwrap()));
    let facts = video.video.as_ref().unwrap();
    assert_eq!((facts.width, facts.height), (1920, 1080));
    assert_eq!(facts.sample_aspect_ratio, Rational::new(1, 1).unwrap());
    assert_eq!(facts.frame_rate, Some(Rational::new(24, 1).unwrap()));
    assert_eq!(facts.cadence, VideoCadence::Unknown);
    assert_eq!(facts.pixel_format, "yuv420p");
    assert_eq!(facts.profile.as_deref(), Some("High"));
    assert_eq!(facts.level, Some(40));
    assert_eq!(video.time_base, Some(Rational::new(1, 12_288).unwrap()));
    assert_eq!(facts.rotation_degrees, -90);

    let audio = selected_audio(&snapshot).unwrap();
    let facts = audio.audio.as_ref().unwrap();
    assert_eq!((facts.sample_rate, facts.channels), (48_000, 2));
    assert_eq!(facts.channel_layout, "stereo");
    assert!(audio.video.is_none());
    assert!(snapshot.streams[2].video.is_none());
    assert!(snapshot.streams[2].audio.is_none());
}

#[test]
fn display_view_uses_exact_snapshot_facts() {
    let snapshot = parse(&complete_output(), auto_stream_intent()).unwrap();
    let display = ProbeDisplay::new(&snapshot).to_string();
    for expected in [
        "Duration:    25/2s",
        "Resolution:  1920x1080",
        "Video codec: h264",
        "Audio codec: aac",
        "Sample rate: 48000 Hz",
    ] {
        assert!(display.contains(expected));
    }

    let disabled = parse(
        &complete_output(),
        intent(StreamChoice::Disabled, StreamChoice::Disabled),
    )
    .unwrap();
    let display = ProbeDisplay::new(&disabled).to_string();
    assert!(display.contains("Audio:       none"));
    assert!(!display.contains("Resolution:"));

    let mut unknown = disabled;
    unknown.container_duration = None;
    assert!(ProbeDisplay::new(&unknown)
        .to_string()
        .contains("Duration:    unknown"));
}

#[test]
fn malformed_json_is_a_typed_error_with_source() {
    let error = parse_ffprobe_json("{", identity(), auto_stream_intent(), FIXTURE_PROBE_ENGINE)
        .unwrap_err();
    assert!(matches!(error, ProbeError::InvalidJson { .. }));
    assert!(error.to_string().contains("failed to parse ffprobe output"));
    assert!(error.source().is_some());

    let wrong_shape =
        parse_ffprobe_json("[]", identity(), auto_stream_intent(), FIXTURE_PROBE_ENGINE)
            .unwrap_err();
    assert!(matches!(wrong_shape, ProbeError::InvalidJson { .. }));
}

#[test]
fn rejects_empty_engine_and_unimplemented_identity_algorithms() {
    let error = parse_ffprobe_json(
        &complete_output().to_string(),
        identity(),
        auto_stream_intent(),
        "  ",
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProbeError::InvalidField {
            field: "engine",
            ..
        }
    ));

    let mut unsupported = identity();
    unsupported.algorithm = HashAlgorithm::Blake3;
    let error = parse_ffprobe_json(
        &complete_output().to_string(),
        unsupported,
        auto_stream_intent(),
        FIXTURE_PROBE_ENGINE,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProbeError::UnsupportedHashAlgorithm {
            algorithm: HashAlgorithm::Blake3
        }
    ));
    assert!(error.to_string().contains("Blake3"));

    let mut malformed = identity();
    malformed.digest = "A".repeat(64);
    let error = parse_ffprobe_json(
        &complete_output().to_string(),
        malformed,
        auto_stream_intent(),
        FIXTURE_PROBE_ENGINE,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProbeError::InvalidField {
            field: "observed_identity.digest",
            ..
        }
    ));

    let mut short = identity();
    short.digest = "abcd".to_owned();
    let error = parse_ffprobe_json(
        &complete_output().to_string(),
        short,
        auto_stream_intent(),
        FIXTURE_PROBE_ENGINE,
    )
    .unwrap_err();
    assert!(matches!(
        error,
        ProbeError::InvalidField {
            field: "observed_identity.digest",
            ..
        }
    ));
}

#[test]
fn display_skips_invalid_external_selection_without_panicking() {
    let mut snapshot = parse(&complete_output(), auto_stream_intent()).unwrap();
    snapshot.selected_video_stream.as_mut().unwrap().type_index = 99;
    assert!(selected_video(&snapshot).is_none());

    snapshot = parse(&complete_output(), auto_stream_intent()).unwrap();
    snapshot.selected_video_stream = snapshot.selected_audio_stream;
    snapshot.selected_audio_stream = Some(veac_ir::StreamSelection {
        global_index: 999,
        type_index: 0,
    });
    assert!(selected_video(&snapshot).is_none());
    assert!(selected_audio(&snapshot).is_none());
    let display = ProbeDisplay::new(&snapshot).to_string();
    assert!(!display.contains("Resolution:"));
    assert!(display.contains("Audio:       none"));
}

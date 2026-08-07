use std::path::Path;

use veac_artifact::*;
use veac_ir::{Rational, RationalTime, StreamSelection};

use super::*;

#[test]
fn every_ffmpeg_spec_uses_bounded_direct_typed_arguments() {
    let specs = [
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream(2),
            source_clock: clock(),
            width: 640,
            height: 360,
            frame_rate: Rational::new(30, 1).unwrap(),
            crf: 23,
        }),
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: stream(3),
            source_clock: clock(),
            sample_rate: 48_000,
            channels: 2,
        }),
        MediaArtifactSpec::Waveform(WaveformSpec {
            source_stream: stream(4),
            source_clock: clock(),
            sample_rate: 24_000,
            width: 800,
            height: 120,
            color: "blue".into(),
        }),
        MediaArtifactSpec::Thumbnail(ThumbnailSpec {
            source_stream: stream(5),
            at: time(125),
            width: 320,
            height: 180,
        }),
        MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
            source_stream: stream(6),
            source_clock: clock(),
            width: 640,
            height: 360,
            frame_rate: Rational::new(60, 1).unwrap(),
            method: OpticalFlowMethod::MotionCompensated,
        }),
        MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
            video_stream: stream(7),
            start: time(10),
            duration: time(50),
            width: 1920,
            height: 1080,
            frame_rate: Rational::new(30, 1).unwrap(),
            audio: Some(SourceSegmentAudioSpec {
                source_stream: stream(8),
                sample_rate: 48_000,
                channels: 2,
            }),
            crf: 18,
        }),
    ];
    for spec in specs {
        let values = args(&spec).unwrap();
        assert_eq!(values[0], "-nostdin");
        assert!(values.iter().any(|value| value == "input name.mp4"));
        assert_eq!(values.last().unwrap(), "out");
        assert_pair(&values, "-protocol_whitelist", "file,pipe");
        assert_pair(&values, "-map_metadata", "-1");
        assert_pair(
            &values,
            "-timelimit",
            &MAX_MEDIA_DERIVATION_CPU_SECONDS.to_string(),
        );
        assert_pair(&values, "-fs", &MAX_ARTIFACT_PAYLOAD_BYTES.to_string());
    }
}

#[test]
fn clocks_filters_and_stream_maps_are_exact() {
    let waveform = MediaArtifactSpec::Waveform(WaveformSpec {
        source_stream: stream(4),
        source_clock: clock(),
        sample_rate: 24_000,
        width: 80,
        height: 20,
        color: "blue".into(),
    });
    let values = args(&waveform).unwrap();
    assert!(values.iter().any(|value| {
        value.contains("[0:4]aresample=24000") && value.contains("aformat=channel_layouts=mono")
    }));
}

#[test]
fn source_segments_map_declared_streams_and_can_disable_audio() {
    let spec = MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
        video_stream: stream(9),
        start: time(0),
        duration: time(100),
        width: 16,
        height: 16,
        frame_rate: Rational::new(12, 1).unwrap(),
        audio: None,
        crf: 20,
    });
    let values = args(&spec).unwrap();
    assert_pair(&values, "-map", "0:9");
    assert_pair(&values, "-r", "12/1");
    assert!(values.iter().any(|value| value == "-an"));
}

#[test]
fn backend_time_rejects_precision_loss_instead_of_rounding() {
    assert_eq!(seconds(time(100)).unwrap(), "1");
    assert_eq!(seconds(time(125)).unwrap(), "1.25");
    assert_eq!(
        seconds(RationalTime::new(1, 3).unwrap()).unwrap_err().kind,
        WorkflowErrorKind::InvalidContract
    );
}

fn args(spec: &MediaArtifactSpec) -> WorkflowResult<Vec<String>> {
    arguments(
        Path::new("input name.mp4"),
        Path::new("out"),
        spec,
        MediaArtifactLimits::default(),
    )
    .map(text)
}

fn assert_pair(values: &[String], name: &str, value: &str) {
    assert!(values
        .windows(2)
        .any(|pair| pair[0] == name && pair[1] == value));
}

fn text(values: Vec<std::ffi::OsString>) -> Vec<String> {
    values
        .into_iter()
        .map(|value| value.into_string().unwrap())
        .collect()
}

fn clock() -> SourceClockSpec {
    SourceClockSpec::Identity {
        duration: time(100),
    }
}

fn stream(global_index: u32) -> StreamSelection {
    StreamSelection {
        global_index,
        type_index: 0,
    }
}

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

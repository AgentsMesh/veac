use serde_json::json;
use veac_ir::{Rational, RationalTime, StreamSelection};

use super::request;
use crate::*;

#[test]
fn invalid_media_specs_fail_before_key_generation() {
    let values = [
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream(),
            source_clock: clock(),
            width: 640,
            height: 360,
            frame_rate: Rational::new(30, 1).unwrap(),
            crf: 52,
        }),
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: stream(),
            source_clock: clock(),
            sample_rate: 0,
            channels: 2,
        }),
        MediaArtifactSpec::Waveform(WaveformSpec {
            source_stream: stream(),
            source_clock: clock(),
            sample_rate: 48_000,
            width: 1,
            height: 0,
            color: "blue".into(),
        }),
        MediaArtifactSpec::Waveform(WaveformSpec {
            source_stream: stream(),
            source_clock: clock(),
            sample_rate: 48_000,
            width: 10,
            height: 10,
            color: "bad color".into(),
        }),
        MediaArtifactSpec::Thumbnail(ThumbnailSpec {
            source_stream: stream(),
            at: RationalTime::new(-1, 10).unwrap(),
            width: 10,
            height: 10,
        }),
        MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
            source_stream: stream(),
            source_clock: clock(),
            width: 10,
            height: 10,
            frame_rate: Rational::new(0, 1).unwrap(),
            method: OpticalFlowMethod::BlockMatching,
        }),
        MediaArtifactSpec::Analysis(AnalysisSpec {
            analysis_type: "analysis".into(),
            configuration: json!([]),
        }),
        MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
            video_stream: stream(),
            start: RationalTime::zero(10).unwrap(),
            duration: RationalTime::zero(10).unwrap(),
            width: 10,
            height: 10,
            frame_rate: Rational::new(30, 1).unwrap(),
            audio: None,
            crf: 18,
        }),
    ];
    for value in values {
        assert_eq!(
            request(value).descriptor().unwrap_err().kind,
            ArtifactErrorKind::InvalidContract
        );
    }
    let mut incomplete = request(MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
        source_stream: stream(),
        source_clock: clock(),
        sample_rate: 48_000,
        channels: 2,
    }));
    incomplete.producer.name.clear();
    assert_eq!(
        incomplete.descriptor().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

fn stream() -> StreamSelection {
    StreamSelection {
        global_index: 0,
        type_index: 0,
    }
}

fn clock() -> SourceClockSpec {
    SourceClockSpec::Identity {
        duration: RationalTime::new(10, 10).unwrap(),
    }
}

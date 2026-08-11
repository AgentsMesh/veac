use veac_ir::{Rational, RationalTime, StreamSelection, TimeRange};

use super::request;
use crate::*;

#[test]
fn request_rejects_invalid_source_and_producer_identity() {
    let spec = audio(clock());
    let mut invalid = request(spec.clone());
    invalid.source_identity.value = "bad".into();
    assert_invalid(invalid);

    let mut invalid = request(spec.clone());
    invalid.producer.version.clear();
    assert_invalid(invalid);

    let mut invalid = request(spec);
    invalid.producer.configuration.value = "bad".into();
    assert_invalid(invalid);
}

#[test]
fn proxy_source_clocks_reject_invalid_identity_and_bounded_domains() {
    let invalid_clocks = [
        SourceClockSpec::Identity {
            duration: raw(1, 0),
        },
        SourceClockSpec::Bounded {
            logical_range: range(-1, 1, 10, 10),
        },
        SourceClockSpec::Bounded {
            logical_range: range(0, 0, 10, 10),
        },
        SourceClockSpec::Bounded {
            logical_range: range(0, 1, 10, 20),
        },
        SourceClockSpec::Bounded {
            logical_range: range(9_007_199_254_740_991, 1, 10, 10),
        },
    ];
    for clock in invalid_clocks {
        assert_invalid(request(audio(clock)));
    }
}

#[test]
fn every_media_spec_rejects_its_remaining_boundary_failures() {
    let specs = [
        MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
            source_stream: stream(),
            source_clock: clock(),
            width: 639,
            height: 360,
            frame_rate: Rational::new(30, 1).unwrap(),
            crf: 20,
        }),
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: stream(),
            source_clock: clock(),
            sample_rate: 48_000,
            channels: 9,
        }),
        MediaArtifactSpec::Waveform(WaveformSpec {
            source_stream: stream(),
            source_clock: clock(),
            sample_rate: 48_000,
            width: 10,
            height: 10,
            color: String::new(),
        }),
        MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
            source_stream: stream(),
            source_clock: clock(),
            width: 31,
            height: 32,
            frame_rate: Rational::new(30, 1).unwrap(),
            method: OpticalFlowMethod::BlockMatching,
        }),
        MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
            video_stream: stream(),
            start: raw(-1, 10),
            duration: raw(1, 10),
            width: 10,
            height: 10,
            frame_rate: Rational::new(30, 1).unwrap(),
            audio: None,
            crf: 18,
        }),
        MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
            video_stream: stream(),
            start: raw(0, 10),
            duration: raw(1, 10),
            width: 0,
            height: 10,
            frame_rate: Rational::new(30, 1).unwrap(),
            audio: None,
            crf: 18,
        }),
        MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
            video_stream: stream(),
            start: raw(0, 10),
            duration: raw(1, 10),
            width: 10,
            height: 10,
            frame_rate: Rational::new(30, 1).unwrap(),
            audio: None,
            crf: 52,
        }),
    ];
    for spec in specs {
        assert_invalid(request(spec));
    }
}

fn audio(source_clock: SourceClockSpec) -> MediaArtifactSpec {
    MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
        source_stream: stream(),
        source_clock,
        sample_rate: 48_000,
        channels: 2,
    })
}

fn assert_invalid(request: MediaArtifactRequest) {
    assert_eq!(
        request.descriptor().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

fn range(start: i64, duration: i64, start_scale: u32, duration_scale: u32) -> TimeRange {
    TimeRange {
        start: raw(start, start_scale),
        duration: raw(duration, duration_scale),
    }
}

fn raw(value: i64, timescale: u32) -> RationalTime {
    RationalTime { value, timescale }
}

fn stream() -> StreamSelection {
    StreamSelection {
        global_index: 0,
        type_index: 0,
    }
}

fn clock() -> SourceClockSpec {
    SourceClockSpec::Identity {
        duration: raw(10, 10),
    }
}

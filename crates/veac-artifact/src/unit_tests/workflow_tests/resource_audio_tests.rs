use veac_ir::{Rational, RationalTime};

use super::{request, stream};
use crate::*;

#[test]
fn sample_rate_and_channel_sample_boundaries_are_exact() {
    let limits = MediaArtifactLimits::default();
    assert_ok(audio(86_400, 48_000, 2), limits);
    assert_limit(audio(86_400, 48_001, 2), limits);
    let rate_limits = MediaArtifactLimits {
        max_sample_rate: 100,
        ..limits
    };
    assert_ok(audio(1, 100, 2), rate_limits);
    assert_limit(audio(1, 101, 2), rate_limits);
}

#[test]
fn pcm_payload_budget_includes_a_conservative_wav_header() {
    let mut limits = MediaArtifactLimits {
        max_sample_rate: 100,
        max_audio_channel_samples: 200,
        max_payload_bytes: 64 * 1024 + 400,
        ..MediaArtifactLimits::default()
    };
    assert_ok(audio(1, 100, 2), limits);
    limits.max_payload_bytes -= 1;
    assert_limit(audio(1, 100, 2), limits);
}

#[test]
fn waveform_and_source_segment_audio_share_the_audio_work_budget() {
    let limits = MediaArtifactLimits {
        max_sample_rate: 100,
        max_audio_channel_samples: 100,
        ..MediaArtifactLimits::default()
    };
    assert_ok(waveform(100), limits);
    assert_limit(waveform(101), limits);
    assert_ok(segment_audio(50, 2), limits);
    assert_limit(segment_audio(51, 2), limits);
}

#[test]
fn resource_policy_cannot_raise_or_disable_any_hard_limit() {
    let hard = MediaArtifactLimits::default();
    let mut values = Vec::new();
    let mut value = hard;
    value.max_dimension = 0;
    values.push(value);
    value = hard;
    value.max_frame_pixels = hard.max_frame_pixels + 1;
    values.push(value);
    value = hard;
    value.max_frame_rate = 0;
    values.push(value);
    value = hard;
    value.max_sample_rate = hard.max_sample_rate + 1;
    values.push(value);
    value = hard;
    value.max_duration_seconds = 0;
    values.push(value);
    value = hard;
    value.max_derivation_cpu_seconds += 1;
    values.push(value);
    value = hard;
    value.max_derivation_wall_seconds = 0;
    values.push(value);
    value = hard;
    value.max_video_frames = 0;
    values.push(value);
    value = hard;
    value.max_pixel_frames += 1;
    values.push(value);
    value = hard;
    value.max_audio_channel_samples = 0;
    values.push(value);
    value = hard;
    value.max_payload_bytes += 1;
    values.push(value);
    value = hard;
    value.max_source_bytes = 0;
    values.push(value);
    for limits in values {
        assert_eq!(
            request(audio(1, 1, 1))
                .validate_with_limits(limits)
                .unwrap_err()
                .kind,
            ArtifactErrorKind::InvalidContract
        );
    }
}

#[test]
fn runtime_limit_tightening_never_changes_the_canonical_artifact_key() {
    let request = request(audio(1, 100, 2));
    let expected = artifact_key(&request.descriptor().unwrap()).unwrap();
    let limits = MediaArtifactLimits {
        max_sample_rate: 100,
        max_audio_channel_samples: 200,
        ..MediaArtifactLimits::default()
    };
    request.validate_with_limits(limits).unwrap();
    assert_eq!(
        artifact_key(&request.descriptor().unwrap()).unwrap(),
        expected
    );
}

fn audio(seconds: i64, sample_rate: u32, channels: u8) -> MediaArtifactSpec {
    MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
        source_stream: stream(),
        source_clock: clock(seconds),
        sample_rate,
        channels,
    })
}

fn waveform(sample_rate: u32) -> MediaArtifactSpec {
    MediaArtifactSpec::Waveform(WaveformSpec {
        source_stream: stream(),
        source_clock: clock(1),
        sample_rate,
        width: 1,
        height: 1,
        color: "blue".into(),
    })
}

fn segment_audio(sample_rate: u32, channels: u8) -> MediaArtifactSpec {
    MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
        video_stream: stream(),
        start: time(0),
        duration: time(1),
        width: 1,
        height: 1,
        frame_rate: Rational::new(1, 1).unwrap(),
        audio: Some(SourceSegmentAudioSpec {
            source_stream: stream(),
            sample_rate,
            channels,
        }),
        crf: 23,
    })
}

fn clock(seconds: i64) -> SourceClockSpec {
    SourceClockSpec::Identity {
        duration: time(seconds),
    }
}

fn time(seconds: i64) -> RationalTime {
    RationalTime::new(seconds, 1).unwrap()
}

fn assert_ok(spec: MediaArtifactSpec, limits: MediaArtifactLimits) {
    request(spec).validate_with_limits(limits).unwrap();
}

fn assert_limit(spec: MediaArtifactSpec, limits: MediaArtifactLimits) {
    assert_eq!(
        request(spec).validate_with_limits(limits).unwrap_err().kind,
        ArtifactErrorKind::ResourceLimit
    );
}

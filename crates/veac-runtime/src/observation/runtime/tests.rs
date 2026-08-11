use veac_ir::{
    HashAlgorithm, MediaIdentity, MediaProbeSnapshot, ProbedStream, ProbedStreamType, Rational,
    RationalTime, StreamDisposition, StreamSelection, VideoCadence, VideoStreamInfo,
};

use super::{selected_video, MediaObserver};
use crate::executor::SystemFfmpeg;
use crate::observation::{FramePixelFormat, FrameRequest, ObservationLimits, ObservationSource};

#[cfg(unix)]
#[path = "tests/ffmpeg.rs"]
mod ffmpeg;

#[test]
fn selected_video_requires_a_selection() {
    let mut snapshot = snapshot();
    snapshot.selected_video_stream = None;

    assert_error(
        selected_video(&snapshot),
        "media observation requires one selected video stream",
    );
}

#[test]
fn selected_video_requires_the_selected_stream() {
    let mut snapshot = snapshot();
    snapshot
        .selected_video_stream
        .as_mut()
        .unwrap()
        .global_index = 9;

    assert_error(
        selected_video(&snapshot),
        "selected observation stream is missing",
    );
}

#[test]
fn selected_video_requires_video_geometry() {
    let mut snapshot = snapshot();
    snapshot.streams[0].video = None;

    assert_error(
        selected_video(&snapshot),
        "selected observation stream has no video geometry",
    );
}

#[test]
fn media_observer_rejects_invalid_limits_and_inexact_time() {
    let limits = ObservationLimits {
        max_wall_seconds: 0,
        ..ObservationLimits::default()
    };
    assert_eq!(
        MediaObserver::new(SystemFfmpeg::default(), limits)
            .unwrap_err()
            .message,
        "invalid media observation resource policy"
    );

    let request = FrameRequest {
        source: ObservationSource {
            path: "unused.mkv".into(),
            identity: snapshot().observed_identity,
            video_stream: None,
        },
        time: RationalTime {
            value: 0,
            timescale: 0,
        },
        pixel_format: FramePixelFormat::Rgb8,
    };
    assert_eq!(
        MediaObserver::default()
            .frame(&request)
            .unwrap_err()
            .message,
        "frame observation time must be exact and non-negative"
    );
}

fn assert_error<T: std::fmt::Debug>(result: Result<T, crate::RuntimeError>, expected: &str) {
    assert_eq!(result.unwrap_err().message, expected);
}

fn snapshot() -> MediaProbeSnapshot {
    MediaProbeSnapshot {
        schema_version: veac_ir::MEDIA_PROBE_SCHEMA_VERSION,
        engine: "test".into(),
        selection_policy: "test".into(),
        container_format: "test".into(),
        container_brand: None,
        observed_identity: MediaIdentity {
            algorithm: HashAlgorithm::Sha256,
            digest: "ab".repeat(32),
        },
        container_duration: None,
        streams: vec![video_stream()],
        selected_video_stream: Some(StreamSelection {
            global_index: 0,
            type_index: 0,
        }),
        selected_audio_stream: None,
    }
}

fn video_stream() -> ProbedStream {
    ProbedStream {
        global_index: 0,
        type_index: 0,
        media_type: ProbedStreamType::Video,
        codec: "h264".into(),
        time_base: Some(Rational::new(1, 1_000).unwrap()),
        start_time: None,
        duration: None,
        disposition: StreamDisposition {
            default: true,
            attached_picture: false,
            timed_thumbnail: false,
        },
        video: Some(VideoStreamInfo {
            width: 320,
            height: 180,
            frame_rate: Some(Rational::new(30, 1).unwrap()),
            cadence: VideoCadence::Constant,
            pixel_format: "yuv420p".into(),
            profile: None,
            level: None,
            sample_aspect_ratio: Rational::new(1, 1).unwrap(),
            rotation_degrees: 0,
        }),
        audio: None,
    }
}

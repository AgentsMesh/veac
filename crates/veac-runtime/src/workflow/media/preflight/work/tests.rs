use veac_artifact::MediaArtifactLimits;
use veac_ir::{AudioStreamInfo, ProbedStreamType, StreamDisposition, VideoStreamInfo};

use super::*;

#[test]
fn valid_video_and_audio_fit_the_default_decode_budget() {
    let limits = MediaArtifactLimits::default();
    assert!(video(&video_stream(), time(1), false, limits).is_ok());
    assert!(video(&video_stream(), time(1), true, limits).is_ok());
    assert!(audio(&audio_stream(), time(1), limits).is_ok());
    assert_eq!(units(time(0), rate(30)).unwrap(), 0);
    assert!(rate_within(rate(30), 60));
    assert!(!rate_within(Rational::new(-1, 1).unwrap(), 60));
}

#[test]
fn video_requires_canonical_timing_and_normalizable_geometry() {
    let limits = MediaArtifactLimits::default();
    let mut stream = video_stream();
    stream.video = None;
    assert_kind(
        video(&stream, time(1), false, limits),
        WorkflowErrorKind::InvalidContract,
    );
    stream = video_stream();
    stream.video.as_mut().unwrap().frame_rate = None;
    assert_kind(
        video(&stream, time(1), false, limits),
        WorkflowErrorKind::InvalidContract,
    );
    stream = video_stream();
    stream.time_base = None;
    assert_limit(video(&stream, time(1), false, limits));
    stream = video_stream();
    stream.video.as_mut().unwrap().frame_rate = Some(rate(i64::from(limits.max_frame_rate) + 1));
    assert_limit(video(&stream, time(1), false, limits));
    stream = video_stream();
    stream.video.as_mut().unwrap().sample_aspect_ratio = Rational::new(0, 1).unwrap();
    assert_kind(
        video(&stream, time(1), false, limits),
        WorkflowErrorKind::InvalidContract,
    );
}

#[test]
fn video_geometry_frame_and_pixel_frame_limits_are_independent() {
    let stream = video_stream();
    for limits in [
        MediaArtifactLimits {
            max_dimension: 100,
            ..MediaArtifactLimits::default()
        },
        MediaArtifactLimits {
            max_frame_pixels: 1,
            ..MediaArtifactLimits::default()
        },
        MediaArtifactLimits {
            max_video_frames: 1,
            ..MediaArtifactLimits::default()
        },
        MediaArtifactLimits {
            max_pixel_frames: 1,
            ..MediaArtifactLimits::default()
        },
    ] {
        assert_limit(video(&stream, time(1), false, limits));
    }
}

#[test]
fn audio_requires_facts_and_stays_within_format_and_work_limits() {
    let limits = MediaArtifactLimits::default();
    let mut stream = audio_stream();
    stream.audio = None;
    assert_kind(
        audio(&stream, time(1), limits),
        WorkflowErrorKind::InvalidContract,
    );
    stream = audio_stream();
    stream.time_base = None;
    assert_limit(audio(&stream, time(1), limits));
    stream = audio_stream();
    stream.audio.as_mut().unwrap().sample_rate = limits.max_sample_rate + 1;
    assert_limit(audio(&stream, time(1), limits));
    stream = audio_stream();
    stream.audio.as_mut().unwrap().channels = 65;
    assert_limit(audio(&stream, time(1), limits));
    let mut tiny = limits;
    tiny.max_audio_channel_samples = 1;
    assert_limit(audio(&audio_stream(), time(1), tiny));
}

fn video_stream() -> ProbedStream {
    ProbedStream {
        global_index: 0,
        type_index: 0,
        media_type: ProbedStreamType::Video,
        codec: "h264".into(),
        time_base: Some(Rational::new(1, 1_000).unwrap()),
        start_time: Some(time(0)),
        duration: Some(time(2)),
        disposition: disposition(),
        video: Some(VideoStreamInfo {
            width: 320,
            height: 180,
            frame_rate: Some(rate(30)),
            pixel_format: "yuv420p".into(),
            profile: None,
            level: None,
            sample_aspect_ratio: rate(1),
            rotation_degrees: 0,
        }),
        audio: None,
    }
}

fn audio_stream() -> ProbedStream {
    ProbedStream {
        global_index: 1,
        type_index: 0,
        media_type: ProbedStreamType::Audio,
        codec: "aac".into(),
        time_base: Some(Rational::new(1, 48_000).unwrap()),
        start_time: Some(time(0)),
        duration: Some(time(2)),
        disposition: disposition(),
        video: None,
        audio: Some(AudioStreamInfo {
            sample_rate: 48_000,
            channels: 2,
            channel_layout: "stereo".into(),
        }),
    }
}

fn disposition() -> StreamDisposition {
    StreamDisposition {
        default: true,
        attached_picture: false,
        timed_thumbnail: false,
    }
}

fn time(seconds: i64) -> RationalTime {
    RationalTime::new(seconds, 1).unwrap()
}

fn rate(value: i64) -> Rational {
    Rational::new(value, 1).unwrap()
}

fn assert_limit(result: WorkflowResult<()>) {
    assert_kind(result, WorkflowErrorKind::ResourceLimit);
}

fn assert_kind(result: WorkflowResult<()>, expected: WorkflowErrorKind) {
    assert_eq!(result.unwrap_err().kind, expected);
}

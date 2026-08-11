use veac_ir::{Rational, RationalTime, TimeRange};

use super::{request, stream};
use crate::*;

#[test]
fn geometry_and_rate_accept_the_boundary_and_reject_the_next_value() {
    let limits = MediaArtifactLimits {
        max_dimension: 100,
        max_frame_pixels: 10_000,
        max_frame_rate: 30,
        ..MediaArtifactLimits::default()
    };
    assert_ok(video(identity(1), 100, 100, 30), limits);
    assert_limit(video(identity(1), 102, 2, 30), limits);
    assert_limit(video(identity(1), 102, 100, 30), limits);
    assert_limit(video(identity(1), 100, 100, 31), limits);
}

#[test]
fn exact_default_frame_and_pixel_work_boundaries_are_enforced() {
    let limits = MediaArtifactLimits::default();
    assert_ok(video(identity(86_400), 2, 2, 30), limits);
    assert_limit(video(identity(86_400), 2, 2, 31), limits);
    assert_ok(video(identity(25_000), 1_600, 1_000, 100), limits);
    assert_limit(video(identity(25_000), 1_602, 1_000, 100), limits);
    assert_ok(video(identity(1), 8_192, 2_048, 1), limits);
}

#[test]
fn clocks_thumbnails_and_segments_bound_the_complete_time_range() {
    let limits = MediaArtifactLimits {
        max_duration_seconds: 10,
        ..MediaArtifactLimits::default()
    };
    let bounded = SourceClockSpec::Bounded {
        logical_range: TimeRange::new(time(9), time(1)).unwrap(),
    };
    assert_ok(video(bounded, 2, 2, 1), limits);
    let too_long = SourceClockSpec::Bounded {
        logical_range: TimeRange::new(time(9), time(2)).unwrap(),
    };
    assert_limit(video(too_long, 2, 2, 1), limits);
    assert_ok(thumbnail(10), limits);
    assert_limit(thumbnail_micros(10_000_001), limits);
    assert_ok(segment(9, 1), limits);
    assert_limit(segment(9, 2), limits);
}

#[test]
fn optical_flow_uses_the_same_bounded_video_work_contract() {
    let limits = MediaArtifactLimits {
        max_video_frames: 30,
        max_pixel_frames: 30_720,
        ..MediaArtifactLimits::default()
    };
    assert_ok(flow(32, 32, 30), limits);
    assert_limit(flow(32, 32, 31), limits);
    assert_limit(flow(34, 32, 30), limits);
}

fn video(clock: SourceClockSpec, width: u32, height: u32, fps: i64) -> MediaArtifactSpec {
    MediaArtifactSpec::ProxyVideo(ProxyVideoSpec {
        source_stream: stream(),
        source_clock: clock,
        width,
        height,
        frame_rate: Rational::new(fps, 1).unwrap(),
        crf: 23,
    })
}

fn flow(width: u32, height: u32, fps: i64) -> MediaArtifactSpec {
    MediaArtifactSpec::OpticalFlow(OpticalFlowSpec {
        source_stream: stream(),
        source_clock: identity(1),
        width,
        height,
        frame_rate: Rational::new(fps, 1).unwrap(),
        method: OpticalFlowMethod::BlockMatching,
    })
}

fn thumbnail(seconds: i64) -> MediaArtifactSpec {
    MediaArtifactSpec::Thumbnail(ThumbnailSpec {
        source_stream: stream(),
        at: time(seconds),
        width: 1,
        height: 1,
    })
}

fn thumbnail_micros(value: i64) -> MediaArtifactSpec {
    MediaArtifactSpec::Thumbnail(ThumbnailSpec {
        source_stream: stream(),
        at: RationalTime::new(value, 1_000_000).unwrap(),
        width: 1,
        height: 1,
    })
}

fn segment(start: i64, duration: i64) -> MediaArtifactSpec {
    MediaArtifactSpec::SourceSegment(SourceSegmentSpec {
        video_stream: stream(),
        start: time(start),
        duration: time(duration),
        width: 2,
        height: 2,
        frame_rate: Rational::new(1, 1).unwrap(),
        audio: None,
        crf: 23,
    })
}

fn identity(seconds: i64) -> SourceClockSpec {
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

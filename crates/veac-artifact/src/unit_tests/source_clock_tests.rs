use veac_ir::{RationalTime, TimeRange, MAX_SAFE_INTEGER};

use crate::{ArtifactErrorKind, SourceClock};

#[test]
fn identity_clock_exposes_its_domain_and_rejects_an_invalid_timescale() {
    let clock = SourceClock::identity(600).unwrap();
    assert_eq!(clock.timescale(), 600);
    assert_eq!(clock.logical_range(), None);
    assert_eq!(clock.physical_start(), time(0, 600));
    assert_eq!(clock.map_point(time(12, 24)), Some(time(12, 24)));
    assert_eq!(clock.map_point(raw(-1, 600)), None);
    assert_eq!(clock.map_point(raw(0, 0)), None);
    assert_eq!(
        SourceClock::identity(0).unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

#[test]
fn bounded_clock_rejects_every_misaligned_or_invalid_domain() {
    let cases = [
        (range(-1, 1, 10, 10), time(0, 10)),
        (range(0, 0, 10, 10), time(0, 10)),
        (range(0, 1, 10, 20), time(0, 10)),
        (range(0, 1, 10, 10), raw(-1, 10)),
        (range(0, 1, 10, 10), time(0, 20)),
        (range(MAX_SAFE_INTEGER as i64, 1, 1, 1), time(0, 1)),
    ];
    for (logical, physical) in cases {
        assert_eq!(
            SourceClock::bounded(logical, physical).unwrap_err().kind,
            ArtifactErrorKind::InvalidContract
        );
    }
}

#[test]
fn bounded_clock_enforces_scale_bounds_and_checked_physical_offsets() {
    let logical = range(10, 5, 10, 10);
    let clock = SourceClock::bounded(logical, time(20, 10)).unwrap();
    assert_eq!(clock.timescale(), 10);
    assert_eq!(clock.logical_range(), Some(logical));
    assert_eq!(clock.physical_start(), time(20, 10));
    assert_eq!(clock.map_point(time(9, 10)), None);
    assert_eq!(clock.map_point(time(10, 20)), None);
    assert_eq!(clock.map_point(time(20, 20)), Some(time(40, 20)));
    assert_eq!(clock.map_point(time(15, 10)), None);
    assert_eq!(clock.map_boundary(time(15, 10)), Some(time(25, 10)));
    assert!(clock.covers_range(range(20, 10, 20, 20)));

    let overflowing =
        SourceClock::bounded(range(0, 1, 1, 1), time(MAX_SAFE_INTEGER as i64, 1)).unwrap();
    assert_eq!(overflowing.map_boundary(time(1, 1)), None);
    assert!(!overflowing.covers_range(range(0, 1, 1, 1)));
    assert!(!clock.covers_range(TimeRange {
        start: time(MAX_SAFE_INTEGER as i64, 1),
        duration: time(1, 1),
    }));
}

#[test]
fn original_stream_clock_normalizes_a_nonzero_physical_start() {
    let clock = SourceClock::original_stream(time(3, 2), Some(time(1, 3))).unwrap();
    assert_eq!(clock.map_point(time(0, 2)), Some(time(2, 6)));
    assert_eq!(clock.map_boundary(time(3, 2)), Some(time(11, 6)));
}

#[test]
fn extending_a_clock_aligns_exact_different_timescales() {
    let clock = SourceClock::bounded(range(0, 3, 2, 2), time(0, 2))
        .unwrap()
        .extended(time(1, 3), time(1, 6))
        .unwrap();
    assert_eq!(clock.logical_range(), Some(range(-2, 12, 6, 6)));
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

fn time(value: i64, timescale: u32) -> RationalTime {
    RationalTime::new(value, timescale).unwrap()
}

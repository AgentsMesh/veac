use super::*;
use veac_artifact::SourceClock;
use veac_plan::canonical::{SourceOutOfRangePolicy, SourceTimeInterpolation, SourceTimeSegment};
use veac_plan::ResolvedSourceTimeMap;

#[test]
fn rejects_invalid_extent_and_boundary_contracts() {
    let (clip, base) = mapping();
    let duration = time(6_000);

    let error = required_error(required(
        &clip,
        &base,
        SourceClock::identity(600).expect("identity clock"),
        None,
        false,
    ));
    assert_eq!(error.diagnostics()[0].code, "SOURCE_BOUNDARY_POLICY");

    let mut mismatched = base.clone();
    linear(
        &mut mismatched,
        RationalTime::new(-1, 1_000).expect("mismatched start"),
        RationalTime::new(1, 1_000).expect("mismatched duration"),
    );
    let error = required_error(required(
        &clip,
        &mismatched,
        SourceClock::identity(600).expect("identity clock"),
        Some(duration),
        false,
    ));
    assert_eq!(error.diagnostics()[0].code, "SOURCE_BOUNDARY_POLICY");
}

#[test]
fn enforces_policy_and_clock_contracts() {
    let (clip, base) = mapping();
    let duration = time(6_000);
    let bounded = SourceClock::bounded(
        TimeRange::new(time(0), duration).expect("clock range"),
        time(0),
    )
    .expect("bounded clock");

    let mut before = base.clone();
    linear(&mut before, time(-1), time(10));
    before.out_of_range = SourceOutOfRangePolicy::Strict;
    assert_eq!(
        required_error(required(&clip, &before, bounded, Some(duration), false)).diagnostics()[0]
            .code,
        "SOURCE_BOUNDARY_POLICY"
    );

    before.out_of_range = SourceOutOfRangePolicy::HoldFirst;
    assert_eq!(
        required_error(required(
            &clip,
            &before,
            SourceClock::identity(600).expect("identity clock"),
            Some(duration),
            false,
        ))
        .diagnostics()[0]
            .code,
        "SOURCE_BOUNDARY_POLICY"
    );

    let late_clock = SourceClock::bounded(
        TimeRange::new(time(1), time(5_999)).expect("late clock"),
        time(0),
    )
    .expect("late clock");
    assert_eq!(
        required_error(required(&clip, &before, late_clock, Some(duration), false,)).diagnostics()
            [0]
        .code,
        "SOURCE_BOUNDARY_POLICY"
    );

    let mut after = base.clone();
    linear(&mut after, time(5_999), time(10));
    after.out_of_range = SourceOutOfRangePolicy::HoldLast;
    let short_clock = SourceClock::bounded(
        TimeRange::new(time(0), time(5_999)).expect("short clock"),
        time(0),
    )
    .expect("short clock");
    assert_eq!(
        required_error(required(&clip, &after, short_clock, Some(duration), false,)).diagnostics()
            [0]
        .code,
        "SOURCE_BOUNDARY_POLICY"
    );
}

#[test]
fn extends_a_hold_point_at_the_exact_source_end() {
    let (clip, mut mapping) = mapping();
    let duration = time(6_000);
    mapping.out_of_range = SourceOutOfRangePolicy::HoldLast;
    mapping.time_map = ResolvedSourceTimeMap::Curve {
        segments: vec![SourceTimeSegment {
            record_duration: time(1),
            source_start: duration,
            source_end: duration,
            interpolation: SourceTimeInterpolation::Hold,
        }],
    };
    let clock = SourceClock::bounded(
        TimeRange::new(time(0), duration).expect("clock range"),
        time(0),
    )
    .expect("clock");
    let padding = required(&clip, &mapping, clock, Some(duration), true)
        .expect("padding calculation")
        .expect("end-point padding");
    assert_eq!(padding.after, time(1));
}

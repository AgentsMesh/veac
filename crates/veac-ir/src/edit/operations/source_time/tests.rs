use crate::test_support::time;

use super::*;

#[test]
fn shift_updates_linear_and_curve_maps_and_rejects_negative_handles() {
    let mut linear = linear(10, 1, PlaybackDirection::Forward);
    shift_source(&mut linear, time(5), "itm").unwrap();
    assert_eq!(linear_start(&linear), time(15));
    let error = shift_source(&mut linear, time(-20), "itm").unwrap_err();
    assert!(error.message.contains("start of media"));

    let mut curve = curve();
    shift_source(&mut curve, time(5), "itm").unwrap();
    let segments = curve_segments(&curve);
    assert_eq!(
        (segments[0].source_start, segments[1].source_end),
        (time(5), time(25))
    );
    let error = shift_source(&mut curve, time(-10), "itm").unwrap_err();
    assert!(error.message.contains("start of media"));
}

#[test]
fn linear_trim_respects_direction_edge_rate_and_repeat() {
    let mut forward_in = linear(100, 2, PlaybackDirection::Forward);
    trim_source(&mut forward_in, TrimEdge::In, time(10), "itm").unwrap();
    assert_eq!(linear_start(&forward_in), time(120));

    let mut forward_out = linear(100, 2, PlaybackDirection::Forward);
    trim_source(&mut forward_out, TrimEdge::Out, time(-10), "itm").unwrap();
    assert_eq!(linear_start(&forward_out), time(100));

    let mut reverse_in = linear(100, 2, PlaybackDirection::Reverse);
    trim_source(&mut reverse_in, TrimEdge::In, time(10), "itm").unwrap();
    assert_eq!(linear_start(&reverse_in), time(100));

    let mut reverse_out = linear(100, 2, PlaybackDirection::Reverse);
    trim_source(&mut reverse_out, TrimEdge::Out, time(-10), "itm").unwrap();
    assert_eq!(linear_start(&reverse_out), time(120));

    let SourceTimeMap::Linear { repeat, .. } = &mut forward_in.time_map else {
        unreachable!()
    };
    *repeat = 2;
    assert!(trim_source(&mut forward_in, TrimEdge::In, time(1), "itm").is_err());

    let mut inexact = SourceMapping::linear(time(0), Rational::new(1, 2).unwrap());
    assert!(trim_source(&mut inexact, TrimEdge::In, time(1), "itm").is_err());
}

#[test]
fn curve_trim_crops_in_and_extrapolates_terminal_hold_out() {
    let mut trim_in = curve();
    trim_source(&mut trim_in, TrimEdge::In, time(5), "itm").unwrap();
    let segments = curve_segments(&trim_in);
    assert_eq!(segments.len(), 2);
    assert_eq!(
        (segments[0].record_duration, segments[0].source_start),
        (time(5), time(10))
    );

    let mut trim_out = curve();
    trim_source(&mut trim_out, TrimEdge::Out, time(5), "itm").unwrap();
    let segments = curve_segments(&trim_out);
    assert_eq!(segments.len(), 2);
    assert_eq!(segments[1].record_duration, time(10));
    assert_eq!(
        (segments[1].source_start, segments[1].source_end),
        (time(20), time(20))
    );

    let invalid = RationalTime {
        value: 1,
        timescale: 0,
    };
    assert!(trim_source(&mut trim_out, TrimEdge::Out, invalid, "itm").is_err());
}

#[test]
fn split_handles_absent_forward_reverse_and_repeated_mappings() {
    let mut none = None;
    let mut some = Some(linear(100, 1, PlaybackDirection::Forward));
    split_source(&mut none, &mut some, time(4), time(6), "itm").unwrap();
    assert_eq!(linear_start(some.as_ref().unwrap()), time(100));

    let mut left = Some(linear(100, 2, PlaybackDirection::Forward));
    let mut right = left.clone();
    split_source(&mut left, &mut right, time(4), time(6), "itm").unwrap();
    assert_eq!(linear_start(left.as_ref().unwrap()), time(100));
    assert_eq!(linear_start(right.as_ref().unwrap()), time(108));

    let mut left = Some(linear(100, 2, PlaybackDirection::Reverse));
    let mut right = left.clone();
    split_source(&mut left, &mut right, time(4), time(6), "itm").unwrap();
    assert_eq!(linear_start(left.as_ref().unwrap()), time(112));
    assert_eq!(linear_start(right.as_ref().unwrap()), time(100));

    let SourceTimeMap::Linear { repeat, .. } = &mut left.as_mut().unwrap().time_map else {
        unreachable!()
    };
    *repeat = 2;
    assert!(split_source(&mut left, &mut right, time(4), time(6), "itm").is_err());
}

#[test]
fn split_rejects_mixed_timebases_before_mutating_mappings() {
    let mut left = Some(linear(0, 1, PlaybackDirection::Forward));
    let mut right = left.clone();
    let other_scale = RationalTime::new(1, 1_000).unwrap();
    assert!(split_source(&mut left, &mut right, time(1), other_scale, "itm").is_err());
    assert_eq!(linear_start(left.as_ref().unwrap()), time(0));
    assert_eq!(linear_start(right.as_ref().unwrap()), time(0));
}

fn linear(start: i64, rate: i64, direction: PlaybackDirection) -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Linear {
            source_start: time(start),
            rate: Rational::new(rate, 1).unwrap(),
            repeat: 1,
            direction,
        },
        frame_synthesis: FrameSynthesisPolicy::Nearest,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}

fn curve() -> SourceMapping {
    SourceMapping {
        time_map: SourceTimeMap::Curve {
            segments: vec![
                segment(10, 0, 20, SourceTimeInterpolation::Linear),
                segment(5, 20, 20, SourceTimeInterpolation::Hold),
            ],
        },
        frame_synthesis: FrameSynthesisPolicy::Blend,
        out_of_range: SourceOutOfRangePolicy::Strict,
    }
}

fn segment(
    duration: i64,
    start: i64,
    end: i64,
    kind: SourceTimeInterpolation,
) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation: kind,
    }
}

fn linear_start(mapping: &SourceMapping) -> RationalTime {
    let SourceTimeMap::Linear { source_start, .. } = mapping.time_map else {
        panic!("linear mapping")
    };
    source_start
}

fn curve_segments(mapping: &SourceMapping) -> &[SourceTimeSegment] {
    let SourceTimeMap::Curve { segments } = &mapping.time_map else {
        panic!("curve mapping")
    };
    segments
}

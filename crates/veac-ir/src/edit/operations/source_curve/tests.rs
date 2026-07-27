use super::*;

fn time(value: i64) -> RationalTime {
    RationalTime::new(value, 100).unwrap()
}

fn segment(
    duration: i64,
    start: i64,
    end: i64,
    interpolation: SourceTimeInterpolation,
) -> SourceTimeSegment {
    SourceTimeSegment {
        record_duration: time(duration),
        source_start: time(start),
        source_end: time(end),
        interpolation,
    }
}

fn crop_curve(
    segments: &[SourceTimeSegment],
    start: RationalTime,
    duration: RationalTime,
    id: &str,
) -> Result<Vec<SourceTimeSegment>, Diagnostic> {
    crop_curve_with_policy(segments, start, duration, false, id)
}

#[test]
fn duration_and_crop_cover_linear_hold_reverse_and_internal_boundaries() {
    let segments = vec![
        segment(10, 0, 20, SourceTimeInterpolation::Linear),
        segment(5, 20, 20, SourceTimeInterpolation::Hold),
        segment(5, 20, 10, SourceTimeInterpolation::Linear),
    ];
    assert_eq!(curve_duration(&segments, "itm").unwrap(), time(20));
    let cropped = crop_curve(&segments, time(5), time(14), "itm").unwrap();
    assert_eq!(cropped.len(), 3);
    assert_eq!(cropped[0].record_duration, time(5));
    assert_eq!(
        (cropped[0].source_start, cropped[0].source_end),
        (time(10), time(20))
    );
    assert_eq!(cropped[1].interpolation, SourceTimeInterpolation::Hold);
    assert_eq!(
        (cropped[1].source_start, cropped[1].source_end),
        (time(20), time(20))
    );
    assert_eq!(
        (cropped[2].source_start, cropped[2].source_end),
        (time(20), time(12))
    );

    let reverse = vec![segment(10, 200, 100, SourceTimeInterpolation::Linear)];
    let cropped = crop_curve(&reverse, time(2), time(6), "itm").unwrap();
    assert_eq!(
        (cropped[0].source_start, cropped[0].source_end),
        (time(180), time(120))
    );
}

#[test]
fn crop_extrapolates_with_the_terminal_segment() {
    let segments = vec![segment(10, 100, 200, SourceTimeInterpolation::Linear)];
    let cropped = crop_curve(&segments, time(10), time(5), "itm").unwrap();
    assert_eq!(cropped.len(), 1);
    assert_eq!(cropped[0].record_duration, time(5));
    assert_eq!(
        (cropped[0].source_start, cropped[0].source_end),
        (time(200), time(250))
    );

    let hold = vec![segment(10, 40, 40, SourceTimeInterpolation::Hold)];
    let cropped = crop_curve(&hold, time(15), time(5), "itm").unwrap();
    assert_eq!(
        (cropped[0].source_start, cropped[0].source_end),
        (time(40), time(40))
    );
}

#[test]
fn empty_zero_and_inexact_crops_are_rejected() {
    assert!(crop_curve(&[], time(0), time(1), "itm").is_err());
    assert!(crop_curve(
        &[segment(1, 0, 1, SourceTimeInterpolation::Linear)],
        time(0),
        time(0),
        "itm",
    )
    .is_err());
    assert!(curve_duration(&[], "itm").is_err());

    let inexact = vec![segment(3, 0, 2, SourceTimeInterpolation::Linear)];
    let error = crop_curve(&inexact, time(1), time(1), "itm").unwrap_err();
    assert!(error.message.contains("not exact"));
}

#[test]
fn crops_reject_negative_source_time_and_arithmetic_overflow() {
    let before_start = vec![segment(10, 5, -5, SourceTimeInterpolation::Linear)];
    let error = crop_curve(&before_start, time(6), time(1), "itm").unwrap_err();
    assert!(error.message.contains("before media start"));

    let maximum = crate::MAX_SAFE_INTEGER as i64;
    let huge = vec![SourceTimeSegment {
        record_duration: time(1),
        source_start: time(0),
        source_end: time(maximum),
        interpolation: SourceTimeInterpolation::Linear,
    }];
    let error = crop_curve(&huge, time(maximum - 1), time(1), "itm").unwrap_err();
    assert!(error.message.contains("overflowed"));
}

#[test]
fn duration_rejects_invalid_and_mixed_timebases() {
    let invalid = SourceTimeSegment {
        record_duration: RationalTime {
            value: 1,
            timescale: 0,
        },
        source_start: time(0),
        source_end: time(1),
        interpolation: SourceTimeInterpolation::Linear,
    };
    assert!(curve_duration(&[invalid], "itm").is_err());

    let mixed = vec![
        segment(1, 0, 1, SourceTimeInterpolation::Linear),
        SourceTimeSegment {
            record_duration: RationalTime::new(1, 200).unwrap(),
            source_start: time(1),
            source_end: time(2),
            interpolation: SourceTimeInterpolation::Linear,
        },
    ];
    assert!(curve_duration(&mixed, "itm").is_err());
}

#[test]
fn segment_lookup_and_interpolation_fail_closed_at_arithmetic_boundaries() {
    let error = segment_at(&[], time(0), "itm").unwrap_err();
    assert!(error.message.contains("curve is empty"));

    let maximum = crate::MAX_SAFE_INTEGER as i64;
    let huge = segment(1, 0, maximum, SourceTimeInterpolation::Linear);
    let error = interpolate(&huge, time(maximum), "itm").unwrap_err();
    assert!(error.message.contains("arithmetic overflowed"));
}

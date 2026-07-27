use std::cmp::Ordering;

use veac_plan::canonical::{
    Rational, RationalTime, SourceOutOfRangePolicy, SourceTimeInterpolation, SourceTimeSegment,
    TimeRange,
};
use veac_plan::{ResolvedClip, ResolvedSourceTimeMap};

use super::Extent;

pub(super) fn extent(
    mapping: &ResolvedSourceTimeMap,
    outside: SourceOutOfRangePolicy,
    clip: &ResolvedClip,
    timebase: u32,
) -> Option<Extent> {
    if !valid_duration(clip.record_range.duration, timebase) {
        return None;
    }
    match mapping {
        ResolvedSourceTimeMap::Linear {
            source_range_per_repeat,
            rate,
            repeat,
            ..
        } => linear(
            *source_range_per_repeat,
            *rate,
            *repeat,
            outside,
            clip,
            timebase,
        ),
        ResolvedSourceTimeMap::Curve { segments } => curve(segments, outside, clip, timebase),
    }
}

pub(super) fn valid_point(value: RationalTime, timebase: u32) -> bool {
    value.is_valid() && value.value >= 0 && value.timescale == timebase
}

fn linear(
    range: TimeRange,
    rate: Rational,
    repeat: u32,
    outside: SourceOutOfRangePolicy,
    clip: &ResolvedClip,
    timebase: u32,
) -> Option<Extent> {
    let end = range.end().ok()?;
    if range.duration.value <= 0
        || !range.start.is_valid()
        || !range.duration.is_valid()
        || range.start.timescale != range.duration.timescale
        || range.start.timescale != timebase
        || !valid_coordinate(range.start, outside, timebase)
        || !valid_coordinate(end, outside, timebase)
        || !rate.is_positive()
        || !(1..=veac_plan::canonical::MAX_SOURCE_REPEAT).contains(&repeat)
        || !duration_matches(range.duration, clip.record_range.duration, rate, repeat)
    {
        return None;
    }
    Some(Extent::Range {
        start: range.start,
        end,
        upper_inclusive: false,
    })
}

fn curve(
    segments: &[SourceTimeSegment],
    outside: SourceOutOfRangePolicy,
    clip: &ResolvedClip,
    timebase: u32,
) -> Option<Extent> {
    let first = segments.first()?;
    let mut total = RationalTime::zero(timebase).ok()?;
    let mut previous = None;
    let mut direction = None;
    let mut minimum = first.source_start;
    let mut maximum = first.source_start;
    let mut maximum_is_point = true;
    for segment in segments {
        let ordering = segment.source_end.partial_cmp(&segment.source_start)?;
        let shape = matches!(
            (segment.interpolation, ordering),
            (SourceTimeInterpolation::Hold, Ordering::Equal)
                | (
                    SourceTimeInterpolation::Linear,
                    Ordering::Less | Ordering::Greater
                )
        );
        if !shape
            || !valid_duration(segment.record_duration, timebase)
            || !valid_coordinate(segment.source_start, outside, timebase)
            || !valid_coordinate(segment.source_end, outside, timebase)
            || previous.is_some_and(|end| end != segment.source_start)
            || direction.is_some_and(|known| ordering != Ordering::Equal && known != ordering)
        {
            return None;
        }
        if ordering != Ordering::Equal {
            direction = Some(ordering);
        }
        total = total.checked_add(segment.record_duration).ok()?;
        previous = Some(segment.source_end);
        for (value, is_point) in [(segment.source_start, true), (segment.source_end, false)] {
            if value.partial_cmp(&minimum)? == Ordering::Less {
                minimum = value;
            }
            match value.partial_cmp(&maximum)? {
                Ordering::Greater => {
                    maximum = value;
                    maximum_is_point = is_point;
                }
                Ordering::Equal => maximum_is_point |= is_point,
                Ordering::Less => {}
            }
        }
    }
    if total != clip.record_range.duration {
        return None;
    }
    if minimum == maximum {
        Some(Extent::Point(minimum))
    } else {
        Some(Extent::Range {
            start: minimum,
            end: maximum,
            upper_inclusive: maximum_is_point,
        })
    }
}

fn valid_coordinate(value: RationalTime, outside: SourceOutOfRangePolicy, timebase: u32) -> bool {
    value.is_valid() && value.timescale == timebase && (value.value >= 0 || outside.allows_before())
}

fn valid_duration(value: RationalTime, timebase: u32) -> bool {
    value.is_valid() && value.value > 0 && value.timescale == timebase
}

fn duration_matches(
    source: RationalTime,
    record: RationalTime,
    rate: Rational,
    repeat: u32,
) -> bool {
    equal_products(
        &[
            source.value as u128,
            u128::from(repeat),
            u128::from(rate.denominator),
            u128::from(record.timescale),
        ],
        &[
            record.value as u128,
            u128::from(source.timescale),
            rate.numerator as u128,
        ],
    )
}

fn equal_products(left: &[u128], right: &[u128]) -> bool {
    let mut left = left.to_vec();
    let mut right = right.to_vec();
    for left_value in &mut left {
        for right_value in &mut right {
            let divisor = gcd(*left_value, *right_value);
            *left_value /= divisor;
            *right_value /= divisor;
        }
    }
    left.into_iter().all(|value| value == 1) && right.into_iter().all(|value| value == 1)
}

fn gcd(mut left: u128, mut right: u128) -> u128 {
    while right != 0 {
        (left, right) = (right, left % right);
    }
    left.max(1)
}

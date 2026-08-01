use veac_ir::{Clip, PlaybackDirection, Rational, RationalTime, TimeRange};

use super::math::{difference, range, scale, sum};

pub(super) fn project(
    clip: &Clip,
    local: &[TimeRange],
    source_start: RationalTime,
    rate: Rational,
    repeat: u32,
    direction: PlaybackDirection,
) -> Vec<TimeRange> {
    let Some(repeat_duration) = scale(clip.record_range.duration, 1, u128::from(repeat)) else {
        return Vec::new();
    };
    let Some(source_span) = scale(
        repeat_duration,
        i128::from(rate.numerator),
        u128::from(rate.denominator),
    ) else {
        return Vec::new();
    };
    let mut output = Vec::new();
    for index in 0..repeat {
        let Some(record_start) = scale(repeat_duration, i128::from(index), 1) else {
            return Vec::new();
        };
        let Some(record_range) = range(record_start, repeat_duration) else {
            return Vec::new();
        };
        for selected in local
            .iter()
            .filter_map(|value| crate::resolver::apply_ranges::intersect(*value, record_range))
        {
            if let Some(value) = project_part(
                selected,
                record_start,
                source_start,
                source_span,
                rate,
                direction,
            ) {
                output.push(value);
            }
        }
    }
    output
}

fn project_part(
    selected: TimeRange,
    repeat_start: RationalTime,
    source_start: RationalTime,
    source_span: RationalTime,
    rate: Rational,
    direction: PlaybackDirection,
) -> Option<TimeRange> {
    let record_offset = difference(selected.start, repeat_start)?;
    let record_end = difference(selected.end().ok()?, repeat_start)?;
    let first = scale(
        record_offset,
        i128::from(rate.numerator),
        u128::from(rate.denominator),
    )?;
    let last = scale(
        record_end,
        i128::from(rate.numerator),
        u128::from(rate.denominator),
    )?;
    let start_offset = match direction {
        PlaybackDirection::Forward => first,
        PlaybackDirection::Reverse => difference(source_span, last)?,
    };
    let start = sum(source_start, start_offset)?;
    range(
        start,
        scale(
            selected.duration,
            i128::from(rate.numerator),
            u128::from(rate.denominator),
        )?,
    )
}

use crate::edit::operation_error;
use crate::*;

use super::time_math::{add, subtract};

#[cfg(test)]
mod tests;

pub(super) fn crop_curve_with_policy(
    segments: &[SourceTimeSegment],
    start: RationalTime,
    duration: RationalTime,
    allow_before: bool,
    id: &str,
) -> Result<Vec<SourceTimeSegment>, Diagnostic> {
    if duration.value <= 0 || segments.is_empty() {
        return Err(operation_error(id, "source-time curve crop is empty"));
    }
    let end = add(start, duration, id)?;
    let mut boundaries = vec![start];
    let mut cursor = zero(start, id)?;
    for segment in segments.iter().take(segments.len() - 1) {
        cursor = add(cursor, segment.record_duration, id)?;
        if cursor > start && cursor < end {
            boundaries.push(cursor);
        }
    }
    boundaries.push(end);
    boundaries
        .windows(2)
        .map(|window| crop_segment(segments, window[0], window[1], allow_before, id))
        .collect()
}

pub(super) fn curve_duration(
    segments: &[SourceTimeSegment],
    id: &str,
) -> Result<RationalTime, Diagnostic> {
    let first = segments
        .first()
        .ok_or_else(|| operation_error(id, "source-time curve is empty"))?;
    segments
        .iter()
        .try_fold(zero(first.record_duration, id)?, |sum, segment| {
            add(sum, segment.record_duration, id)
        })
}

fn crop_segment(
    segments: &[SourceTimeSegment],
    start: RationalTime,
    end: RationalTime,
    allow_before: bool,
    id: &str,
) -> Result<SourceTimeSegment, Diagnostic> {
    let (segment, segment_start) = segment_at(segments, start, id)?;
    let source_start = interpolate(segment, subtract(start, segment_start, id)?, id)?;
    let source_end = interpolate(segment, subtract(end, segment_start, id)?, id)?;
    if !allow_before {
        nonnegative(source_start, id)?;
        nonnegative(source_end, id)?;
    }
    Ok(SourceTimeSegment {
        record_duration: subtract(end, start, id)?,
        source_start,
        source_end,
        interpolation: segment.interpolation,
    })
}

fn segment_at<'a>(
    segments: &'a [SourceTimeSegment],
    time: RationalTime,
    id: &str,
) -> Result<(&'a SourceTimeSegment, RationalTime), Diagnostic> {
    let mut start = zero(time, id)?;
    for segment in segments {
        let end = add(start, segment.record_duration, id)?;
        if time < end {
            return Ok((segment, start));
        }
        start = end;
    }
    let last = segments
        .last()
        .ok_or_else(|| operation_error(id, "source-time curve is empty"))?;
    Ok((last, subtract(start, last.record_duration, id)?))
}

fn interpolate(
    segment: &SourceTimeSegment,
    elapsed: RationalTime,
    id: &str,
) -> Result<RationalTime, Diagnostic> {
    if segment.interpolation == SourceTimeInterpolation::Hold {
        return Ok(segment.source_start);
    }
    let source_delta = subtract(segment.source_end, segment.source_start, id)?;
    let numerator = i128::from(source_delta.value) * i128::from(elapsed.value);
    let denominator = i128::from(segment.record_duration.value);
    if denominator == 0 || numerator % denominator != 0 {
        return Err(operation_error(
            id,
            "source-time curve edit is not exact at the project timebase",
        ));
    }
    let value = i64::try_from(numerator / denominator)
        .map_err(|_| operation_error(id, "source-time curve arithmetic overflowed"))?;
    let delta = RationalTime::new(value, elapsed.timescale)
        .map_err(|_| operation_error(id, "source-time curve arithmetic overflowed"))?;
    add(segment.source_start, delta, id)
}

fn zero(time: RationalTime, id: &str) -> Result<RationalTime, Diagnostic> {
    RationalTime::zero(time.timescale).map_err(|_| operation_error(id, "invalid source timebase"))
}

fn nonnegative(time: RationalTime, id: &str) -> Result<(), Diagnostic> {
    (time.value >= 0)
        .then_some(())
        .ok_or_else(|| operation_error(id, "source-time edit would move before media start"))
}

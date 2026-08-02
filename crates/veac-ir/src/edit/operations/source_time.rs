use crate::edit::operation_error;
use crate::*;

use super::source_curve::{crop_curve_with_policy, curve_duration};
use super::time_math::{add, scale, subtract};

#[cfg(test)]
mod tests;

pub(super) fn shift_source(
    mapping: &mut SourceMapping,
    delta: RationalTime,
    object_id: &str,
) -> Result<(), Diagnostic> {
    let allow_before = mapping.out_of_range.allows_before();
    match &mut mapping.time_map {
        SourceTimeMap::Linear { source_start, .. } => {
            *source_start = add(*source_start, delta, object_id)?;
            nonnegative(*source_start, allow_before, object_id)
        }
        SourceTimeMap::Curve { segments } => {
            for segment in segments {
                segment.source_start = add(segment.source_start, delta, object_id)?;
                segment.source_end = add(segment.source_end, delta, object_id)?;
                nonnegative(segment.source_start, allow_before, object_id)?;
                nonnegative(segment.source_end, allow_before, object_id)?;
            }
            Ok(())
        }
    }
}

pub(super) fn trim_source(
    mapping: &mut SourceMapping,
    edge: TrimEdge,
    delta: RationalTime,
    object_id: &str,
) -> Result<(), Diagnostic> {
    let allow_before = mapping.out_of_range.allows_before();
    match &mut mapping.time_map {
        SourceTimeMap::Linear {
            source_start,
            rate,
            repeat,
            direction,
        } => {
            single_repeat(*repeat, object_id)?;
            let advance = matches!(
                (edge, direction),
                (TrimEdge::In, PlaybackDirection::Forward)
                    | (TrimEdge::Out, PlaybackDirection::Reverse)
            );
            if advance {
                let delta = if edge == TrimEdge::Out {
                    negate(delta, object_id)?
                } else {
                    delta
                };
                *source_start = add(*source_start, scale(delta, *rate, object_id)?, object_id)?;
                nonnegative(*source_start, allow_before, object_id)?;
            }
            Ok(())
        }
        SourceTimeMap::Curve { segments } => {
            let old_duration = curve_duration(segments, object_id)?;
            let start = if edge == TrimEdge::In {
                delta
            } else {
                RationalTime::zero(delta.timescale)
                    .map_err(|_| operation_error(object_id, "invalid source timebase"))?
            };
            let duration = match edge {
                TrimEdge::In => subtract(old_duration, delta, object_id)?,
                TrimEdge::Out => add(old_duration, delta, object_id)?,
            };
            *segments = crop_curve_with_policy(segments, start, duration, allow_before, object_id)?;
            Ok(())
        }
    }
}

pub(super) fn split_source(
    left: &mut Option<SourceMapping>,
    right: &mut Option<SourceMapping>,
    left_duration: RationalTime,
    right_duration: RationalTime,
    object_id: &str,
) -> Result<(), Diagnostic> {
    let (Some(left), Some(right)) = (left, right) else {
        return Ok(());
    };
    let total = add(left_duration, right_duration, object_id)?;
    crop_mapping(
        left,
        zero(total, object_id)?,
        left_duration,
        total,
        object_id,
    )?;
    crop_mapping(right, left_duration, right_duration, total, object_id)
}

fn crop_mapping(
    mapping: &mut SourceMapping,
    start: RationalTime,
    duration: RationalTime,
    old_duration: RationalTime,
    id: &str,
) -> Result<(), Diagnostic> {
    let allow_before = mapping.out_of_range.allows_before();
    match &mut mapping.time_map {
        SourceTimeMap::Linear {
            source_start,
            rate,
            repeat,
            direction,
        } => {
            single_repeat(*repeat, id)?;
            let offset = match direction {
                PlaybackDirection::Forward => start,
                PlaybackDirection::Reverse => {
                    subtract(subtract(old_duration, start, id)?, duration, id)?
                }
            };
            *source_start = add(*source_start, scale(offset, *rate, id)?, id)?;
            nonnegative(*source_start, allow_before, id)
        }
        SourceTimeMap::Curve { segments } => {
            *segments = crop_curve_with_policy(segments, start, duration, allow_before, id)?;
            Ok(())
        }
    }
}

fn single_repeat(repeat: u32, id: &str) -> Result<(), Diagnostic> {
    (repeat == 1).then_some(()).ok_or_else(|| {
        operation_error(
            id,
            "partial edits of repeated media cannot preserve source continuity",
        )
    })
}

fn negate(time: RationalTime, id: &str) -> Result<RationalTime, Diagnostic> {
    subtract(zero(time, id)?, time, id)
}

fn zero(time: RationalTime, id: &str) -> Result<RationalTime, Diagnostic> {
    RationalTime::zero(time.timescale).map_err(|_| operation_error(id, "invalid source timebase"))
}

fn nonnegative(time: RationalTime, allow_before: bool, id: &str) -> Result<(), Diagnostic> {
    (time.value >= 0 || allow_before)
        .then_some(())
        .ok_or_else(|| operation_error(id, "source-time edit would move before the start of media"))
}

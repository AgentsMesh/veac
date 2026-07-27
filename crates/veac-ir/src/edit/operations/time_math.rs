use crate::edit::{ChangeSet, MarkChanged};
use crate::*;

use crate::edit::operation_error;

#[cfg(test)]
mod tests;

pub(super) use super::source_time::{split_source, trim_source};

pub(super) fn add(
    left: RationalTime,
    right: RationalTime,
    object_id: &str,
) -> Result<RationalTime, Diagnostic> {
    left.checked_add(right)
        .map_err(|_| operation_error(object_id, "time arithmetic is invalid or overflowed"))
}

pub(super) fn subtract(
    left: RationalTime,
    right: RationalTime,
    object_id: &str,
) -> Result<RationalTime, Diagnostic> {
    let value = right
        .value
        .checked_neg()
        .ok_or_else(|| operation_error(object_id, "time arithmetic overflowed"))?;
    let negative = RationalTime::new(value, right.timescale)
        .map_err(|_| operation_error(object_id, "time arithmetic is invalid"))?;
    add(left, negative, object_id)
}

pub(super) fn scale(
    time: RationalTime,
    rate: Rational,
    object_id: &str,
) -> Result<RationalTime, Diagnostic> {
    let numerator = i128::from(time.value)
        .checked_mul(i128::from(rate.numerator))
        .ok_or_else(|| operation_error(object_id, "source-time arithmetic overflowed"))?;
    let denominator = i128::from(rate.denominator);
    if numerator % denominator != 0 {
        return Err(operation_error(
            object_id,
            "source-time result is not exact at the project timebase",
        ));
    }
    let value = i64::try_from(numerator / denominator)
        .map_err(|_| operation_error(object_id, "source-time arithmetic overflowed"))?;
    RationalTime::new(value, time.timescale)
        .map_err(|_| operation_error(object_id, "source-time result is outside the safe range"))
}

pub(super) fn shift_from(
    track: &mut Track,
    threshold: RationalTime,
    delta: RationalTime,
    excluded: &[&ItemId],
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    for clip in &mut track.clips {
        if clip.record_range.start >= threshold && !excluded.contains(&&clip.id) {
            clip.record_range.start = add(clip.record_range.start, delta, clip.id.as_str())?;
            changed.item(clip.id.clone());
        }
    }
    Ok(())
}

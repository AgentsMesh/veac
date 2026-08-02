use crate::edit::operation_error;
use crate::*;

#[cfg(test)]
#[path = "range/tests.rs"]
mod tests;

pub(super) fn item_windows(
    project: &Project,
    sequence_id: &SequenceId,
    left_id: &ItemId,
    right_id: &ItemId,
) -> Result<[TimeRange; 2], Diagnostic> {
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == *sequence_id)
        .ok_or_else(|| operation_error(sequence_id.as_str(), "sequence does not exist"))?;
    let find = |id: &ItemId| {
        sequence
            .tracks
            .iter()
            .flat_map(|track| &track.clips)
            .find(|clip| clip.id == *id)
            .ok_or_else(|| operation_error(id.as_str(), "split fragment does not exist"))
    };
    let left = find(left_id)?;
    let right = find(right_id)?;
    let zero = RationalTime::new(0, left.record_range.start.timescale).unwrap();
    let offset = RationalTime::new(
        right.record_range.start.value - left.record_range.start.value,
        left.record_range.start.timescale,
    )
    .map_err(|_| operation_error(right_id.as_str(), "split fragment timebase mismatch"))?;
    Ok([
        TimeRange::new(zero, left.record_range.duration).unwrap(),
        TimeRange::new(offset, right.record_range.duration).unwrap(),
    ])
}

pub(super) fn intersect(range: &TimeRange, window: &TimeRange) -> Option<TimeRange> {
    let start = if range.start > window.start {
        range.start
    } else {
        window.start
    };
    let range_end = range.end().ok()?;
    let window_end = window.end().ok()?;
    let end = if range_end < window_end {
        range_end
    } else {
        window_end
    };
    if start >= end {
        return None;
    }
    let duration = RationalTime::new(end.value - start.value, start.timescale).ok()?;
    TimeRange::new(start, duration).ok()
}

pub(super) fn rebase(mut range: TimeRange, offset: RationalTime) -> TimeRange {
    range.start = RationalTime::new(range.start.value - offset.value, offset.timescale).unwrap();
    range
}

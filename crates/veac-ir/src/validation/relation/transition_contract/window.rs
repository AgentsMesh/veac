use crate::{Clip, RationalTime, RelationItem, TimeRange};

pub(super) fn exact(from: &Clip, to: &Clip) -> Option<TimeRange> {
    let from_start = from.record_range.start;
    let to_start = to.record_range.start;
    if from_start.timescale != to_start.timescale
        || from.record_range.duration.timescale != from_start.timescale
        || to.record_range.duration.timescale != from_start.timescale
    {
        return None;
    }
    let from_end = from.record_range.end().ok()?;
    let to_end = to.record_range.end().ok()?;
    if !(from_start < to_start && to_start < from_end && from_end < to_end) {
        return None;
    }
    Some(TimeRange {
        start: to_start,
        duration: RationalTime {
            value: from_end.value.checked_sub(to_start.value)?,
            timescale: from_start.timescale,
        },
    })
}

pub(super) fn third_item_crosses(
    from: RelationItem<'_>,
    to: RelationItem<'_>,
    window: TimeRange,
) -> bool {
    from.track
        .clips
        .iter()
        .enumerate()
        .filter(|(position, _)| *position != from.position && *position != to.position)
        .any(|(_, clip)| intersects(clip.record_range, window))
}

pub(super) fn intersects(left: TimeRange, right: TimeRange) -> bool {
    left.end()
        .ok()
        .zip(right.end().ok())
        .is_some_and(|(left_end, right_end)| left.start < right_end && right.start < left_end)
}

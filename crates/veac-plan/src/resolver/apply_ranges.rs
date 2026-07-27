use std::cmp::Ordering;

use veac_ir::{RationalTime, TimeRange};

pub(super) fn intersect(left: TimeRange, right: TimeRange) -> Option<TimeRange> {
    let start = maximum(left.start, right.start)?;
    let end = minimum(left.end().ok()?, right.end().ok()?)?;
    (start < end).then(|| between(start, end)).flatten()
}

pub(super) fn absolute(owner: TimeRange, local: Option<TimeRange>) -> Option<TimeRange> {
    let Some(local) = local else {
        return Some(owner);
    };
    let start = owner.start.checked_add(local.start).ok()?;
    let translated = TimeRange::new(start, local.duration).ok()?;
    intersect(owner, translated)
}

pub(super) fn merge(mut ranges: Vec<TimeRange>) -> Vec<TimeRange> {
    ranges.sort_by(|left, right| {
        left.start
            .partial_cmp(&right.start)
            .unwrap_or(Ordering::Equal)
    });
    let mut merged: Vec<TimeRange> = Vec::with_capacity(ranges.len());
    for range in ranges {
        let Some(previous) = merged.last_mut() else {
            merged.push(range);
            continue;
        };
        let Some(previous_end) = previous.end().ok() else {
            continue;
        };
        let Some(range_end) = range.end().ok() else {
            continue;
        };
        if range.start <= previous_end {
            if range_end > previous_end {
                *previous = between(previous.start, range_end).expect("validated range");
            }
        } else {
            merged.push(range);
        }
    }
    merged
}

fn between(start: RationalTime, end: RationalTime) -> Option<TimeRange> {
    super::time::range_between(start, end).filter(|range| range.duration.value > 0)
}

fn maximum(left: RationalTime, right: RationalTime) -> Option<RationalTime> {
    left.partial_cmp(&right)
        .map(|order| if order.is_lt() { right } else { left })
}

fn minimum(left: RationalTime, right: RationalTime) -> Option<RationalTime> {
    left.partial_cmp(&right)
        .map(|order| if order.is_gt() { right } else { left })
}

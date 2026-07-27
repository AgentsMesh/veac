use crate::{ApplyId, TimeRange};

pub(super) struct BandScope<'a> {
    pub id: &'a ApplyId,
    pub range: TimeRange,
    pub from_order: i32,
    pub through_order: i32,
}

pub(super) fn crossing_bands<'a>(bands: &'a [BandScope<'a>]) -> Vec<&'a ApplyId> {
    let mut invalid = Vec::new();
    for (index, left) in bands.iter().enumerate() {
        for right in &bands[index + 1..] {
            if time_overlaps(left.range, right.range) && spatially_crosses(left, right) {
                invalid.push(right.id);
            }
        }
    }
    invalid
}

fn spatially_crosses(left: &BandScope<'_>, right: &BandScope<'_>) -> bool {
    crosses(
        left.from_order,
        left.through_order,
        right.from_order,
        right.through_order,
    ) || crosses(
        right.from_order,
        right.through_order,
        left.from_order,
        left.through_order,
    )
}

fn crosses(left_from: i32, left_through: i32, right_from: i32, right_through: i32) -> bool {
    left_from < right_from && right_from <= left_through && left_through < right_through
}

fn time_overlaps(left: TimeRange, right: TimeRange) -> bool {
    left.end().is_ok_and(|left_end| {
        right
            .end()
            .is_ok_and(|right_end| left.start < right_end && right.start < left_end)
    })
}

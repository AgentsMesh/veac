use veac_plan::canonical::{ItemId, TimeRange, TrackId, TrackKind};
use veac_plan::{ResolvedApply, ResolvedApplyTarget, ResolvedSequence};

use super::Check;

#[cfg(test)]
#[path = "../../unit_tests/apply_target_internal_tests.rs"]
mod internal_tests;

pub(super) fn validate(check: &mut Check, sequence: &ResolvedSequence, apply: &ResolvedApply) {
    let valid = match &apply.target {
        ResolvedApplyTarget::CompositeBand {
            from_track_id,
            through_track_id,
            track_ids,
            active_ranges,
        } => band_valid(
            sequence,
            from_track_id,
            through_track_id,
            track_ids,
            active_ranges,
            apply.record_range,
        ),
        ResolvedApplyTarget::Layer {
            track_id,
            item_ids,
            active_ranges,
        } => layer_valid(
            sequence,
            track_id,
            item_ids,
            active_ranges,
            apply.record_range,
        ),
        ResolvedApplyTarget::ItemSet { items } => {
            !items.is_empty()
                && items
                    .windows(2)
                    .all(|pair| pair[0].item_id < pair[1].item_id)
                && items.iter().all(|item| {
                    sequence
                        .tracks
                        .iter()
                        .find(|track| track.id == item.track_id)
                        .and_then(|track| {
                            track
                                .clips
                                .iter()
                                .find(|clip| clip.id == item.item_id && clip.visual.is_some())
                        })
                        .is_some_and(|clip| {
                            intersection(apply.record_range, clip.record_range)
                                .is_some_and(|range| item.active_range == range)
                        })
                })
        }
    };
    if !valid {
        check.push(
            "PLAN_APPLY_TARGET_INVALID",
            Some(apply.id.to_string()),
            "apply target does not exactly match resolved track and item ownership",
        );
    }
}

pub(super) fn interval(
    sequence: &ResolvedSequence,
    apply: &ResolvedApply,
) -> Option<(usize, usize)> {
    let (from, through) = match &apply.target {
        ResolvedApplyTarget::CompositeBand {
            from_track_id,
            through_track_id,
            ..
        } => (from_track_id, through_track_id),
        ResolvedApplyTarget::Layer { track_id, .. } => (track_id, track_id),
        ResolvedApplyTarget::ItemSet { .. } => return None,
    };
    Some((position(sequence, from)?, position(sequence, through)?))
}

pub(super) fn contains_item(apply: &ResolvedApply, track: &TrackId, item: &ItemId) -> bool {
    match &apply.target {
        ResolvedApplyTarget::CompositeBand { track_ids, .. } => track_ids.contains(track),
        ResolvedApplyTarget::Layer { track_id, .. } => track_id == track,
        ResolvedApplyTarget::ItemSet { items } => items.iter().any(|entry| entry.item_id == *item),
    }
}

fn band_valid(
    sequence: &ResolvedSequence,
    from: &TrackId,
    through: &TrackId,
    actual: &[TrackId],
    ranges: &[TimeRange],
    owner: TimeRange,
) -> bool {
    let Some(from_index) = position(sequence, from) else {
        return false;
    };
    let Some(through_index) = position(sequence, through) else {
        return false;
    };
    if from_index >= through_index {
        return false;
    }
    let expected: Vec<_> = sequence.tracks[from_index..=through_index]
        .iter()
        .filter(|track| visual_track(track.kind) && track.state.visual_enabled)
        .map(|track| track.id.clone())
        .collect();
    actual == expected && ranges_valid(ranges, owner)
}

fn layer_valid(
    sequence: &ResolvedSequence,
    id: &TrackId,
    actual: &[ItemId],
    ranges: &[TimeRange],
    owner: TimeRange,
) -> bool {
    let Some(track) = sequence.tracks.iter().find(|track| track.id == *id) else {
        return false;
    };
    let expected: Vec<_> = track
        .clips
        .iter()
        .filter(|clip| clip.visual.is_some())
        .map(|clip| clip.id.clone())
        .collect();
    visual_track(track.kind) && actual == expected && ranges_valid(ranges, owner)
}

fn ranges_valid(ranges: &[TimeRange], owner: TimeRange) -> bool {
    !ranges.is_empty()
        && ranges.iter().all(|range| within(owner, *range))
        && ranges
            .windows(2)
            .all(|pair| pair[0].end().ok() < Some(pair[1].start))
}

fn intersection(left: TimeRange, right: TimeRange) -> Option<TimeRange> {
    let start = if left.start > right.start {
        left.start
    } else {
        right.start
    };
    let left_end = left.end().expect("preflight timebase");
    let right_end = right.end().expect("preflight timebase");
    let end = if left_end < right_end {
        left_end
    } else {
        right_end
    };
    if start >= end || start.timescale != end.timescale {
        return None;
    }
    TimeRange::new(
        start,
        veac_plan::canonical::RationalTime {
            value: end.value - start.value,
            timescale: start.timescale,
        },
    )
    .ok()
}

fn within(owner: TimeRange, child: TimeRange) -> bool {
    child.duration.value > 0 && owner.start <= child.start && child.end().ok() <= owner.end().ok()
}

fn position(sequence: &ResolvedSequence, id: &TrackId) -> Option<usize> {
    sequence.tracks.iter().position(|track| track.id == *id)
}

fn visual_track(kind: TrackKind) -> bool {
    matches!(
        kind,
        TrackKind::Video | TrackKind::Visual | TrackKind::Caption
    )
}

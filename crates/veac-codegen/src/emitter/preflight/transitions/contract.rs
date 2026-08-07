use veac_plan::canonical::{RationalTime, TimeRange, TrackKind, TransitionAlignment};
use veac_plan::{ResolvedClip, ResolvedTransition};

#[allow(clippy::too_many_arguments)]
pub(super) fn valid(
    value: &ResolvedTransition,
    outgoing: &ResolvedClip,
    incoming: &ResolvedClip,
    track_clips: &[ResolvedClip],
    track_kind: TrackKind,
    visual: bool,
    timebase: u32,
    sequence_duration: RationalTime,
) -> bool {
    if value.alignment != TransitionAlignment::Centered
        || !matches!(track_kind, TrackKind::Video | TrackKind::Visual)
        || !visual
        || outgoing.visual.is_none()
        || incoming.visual.is_none()
    {
        return false;
    }
    let Some(overlap) = exact_overlap(outgoing, incoming, timebase) else {
        return false;
    };
    let expected_cut = overlap.start.value.checked_add(overlap.duration.value / 2);
    let outgoing_start = overlap
        .start
        .value
        .checked_sub(outgoing.record_range.start.value);
    positive(overlap.duration, timebase)
        && value.record_window == overlap
        && expected_cut == Some(value.cut_time.value)
        && valid_time(value.cut_time, timebase)
        && value.outgoing_range == local_range(outgoing_start, overlap.duration, timebase)
        && value.incoming_range == local_range(Some(0), overlap.duration, timebase)
        && overlap.end().is_ok_and(|end| end <= sequence_duration)
        && !third_crosses(track_clips, outgoing, incoming, overlap)
}

fn exact_overlap(
    outgoing: &ResolvedClip,
    incoming: &ResolvedClip,
    timebase: u32,
) -> Option<TimeRange> {
    let from_start = outgoing.record_range.start;
    let to_start = incoming.record_range.start;
    if !valid_time(from_start, timebase) || !valid_time(to_start, timebase) {
        return None;
    }
    let from_end = outgoing.record_range.end().ok()?;
    let to_end = incoming.record_range.end().ok()?;
    if !(from_start < to_start && to_start < from_end && from_end < to_end) {
        return None;
    }
    Some(TimeRange {
        start: to_start,
        duration: RationalTime {
            value: from_end.value.checked_sub(to_start.value)?,
            timescale: timebase,
        },
    })
}

fn local_range(start: Option<i64>, duration: RationalTime, timebase: u32) -> TimeRange {
    TimeRange {
        start: RationalTime {
            value: start.unwrap_or(-1),
            timescale: timebase,
        },
        duration,
    }
}

fn third_crosses(
    clips: &[ResolvedClip],
    outgoing: &ResolvedClip,
    incoming: &ResolvedClip,
    overlap: TimeRange,
) -> bool {
    clips.iter().any(|clip| {
        clip.id != outgoing.id && clip.id != incoming.id && intersects(clip.record_range, overlap)
    })
}

pub(super) fn intersects(left: TimeRange, right: TimeRange) -> bool {
    left.end()
        .ok()
        .zip(right.end().ok())
        .is_some_and(|(left_end, right_end)| left.start < right_end && right.start < left_end)
}

fn positive(value: RationalTime, timebase: u32) -> bool {
    valid_time(value, timebase) && value.value > 0
}

fn valid_time(value: RationalTime, timebase: u32) -> bool {
    value.is_valid() && value.timescale == timebase && value.value >= 0
}

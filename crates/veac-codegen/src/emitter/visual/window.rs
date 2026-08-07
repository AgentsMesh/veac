use veac_plan::canonical::{RationalTime, TimeRange};
use veac_plan::{ResolvedClip, ResolvedTrack};

use super::super::{time, EmitContext};

pub(super) fn normal(track: &ResolvedTrack, clip: &ResolvedClip) -> Option<TimeRange> {
    let mut start = clip.record_range.start;
    let mut end = clip.record_range.end().ok()?;
    for transition in &track.transitions {
        if transition.incoming_clip_id == clip.id {
            start = later(start, transition.record_window.end().ok()?);
        }
        if transition.outgoing_clip_id == clip.id {
            end = earlier(end, transition.record_window.start);
        }
    }
    if start >= end || start.timescale != end.timescale {
        return None;
    }
    let duration = RationalTime::new(end.value.checked_sub(start.value)?, start.timescale).ok()?;
    TimeRange::new(start, duration).ok()
}

pub(super) fn slice(
    context: &mut EmitContext<'_>,
    input: &str,
    clip: &ResolvedClip,
    window: TimeRange,
    prefix: &str,
) -> String {
    if window == clip.record_range {
        return input.to_owned();
    }
    context.graph.filter(
        &[input],
        format!(
            "trim=start={}:duration={},setpts=PTS-STARTPTS",
            time::seconds_delta(clip.record_range.start, window.start),
            time::seconds(window.duration)
        ),
        prefix,
    )
}

fn later(left: RationalTime, right: RationalTime) -> RationalTime {
    if left < right {
        right
    } else {
        left
    }
}

fn earlier(left: RationalTime, right: RationalTime) -> RationalTime {
    if left < right {
        left
    } else {
        right
    }
}

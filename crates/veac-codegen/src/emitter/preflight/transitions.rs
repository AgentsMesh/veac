use std::collections::BTreeSet;

use veac_plan::canonical::{BlendMode, RationalTime, TransitionAlignment};
use veac_plan::{ResolvedClip, ResolvedSequence, ResolvedTransition};

use super::Check;

pub(super) fn validate(check: &mut Check, sequence: &ResolvedSequence, timebase: u32) {
    for track in &sequence.tracks {
        let mut pairs = BTreeSet::new();
        for transition in &track.transitions {
            let pair = (&transition.outgoing_clip_id, &transition.incoming_clip_id);
            let endpoints = track.clips.windows(2).find(|clips| {
                clips[0].id == transition.outgoing_clip_id
                    && clips[1].id == transition.incoming_clip_id
            });
            if track.state.visual_enabled
                && endpoints.is_some_and(|clips| {
                    clips.iter().any(|clip| {
                        clip.visual.as_ref().is_some_and(|visual| {
                            visual.compositing.blend_mode != BlendMode::Normal
                        })
                    })
                })
            {
                check.push(
                    "PLAN_TRANSITION_BLEND_UNSUPPORTED",
                    Some(transition.outgoing_clip_id.to_string()),
                    "visual transition endpoints must use normal blend mode",
                );
            }
            if track.state.visual_enabled
                && endpoints.is_some_and(|clips| {
                    clips[0]
                        .visual
                        .as_ref()
                        .map(|visual| visual.compositing.z_index)
                        != clips[1]
                            .visual
                            .as_ref()
                            .map(|visual| visual.compositing.z_index)
                })
            {
                check.push(
                    "PLAN_TRANSITION_Z_ORDER_UNSUPPORTED",
                    Some(transition.outgoing_clip_id.to_string()),
                    "visual transition endpoints must have equal z-index",
                );
            }
            if !pairs.insert(pair)
                || endpoints.is_none_or(|clips| {
                    !valid(
                        transition,
                        &clips[0],
                        &clips[1],
                        track.state.visual_enabled,
                        track.state.audio_enabled,
                        timebase,
                        sequence.duration,
                    )
                })
            {
                check.push(
                    "PLAN_TRANSITION_INVALID",
                    Some(transition.outgoing_clip_id.to_string()),
                    "transition topology, timing, or endpoint handles are invalid",
                );
            }
        }
        validate_window_overlap(check, &track.transitions);
    }
}

fn valid(
    value: &ResolvedTransition,
    outgoing: &ResolvedClip,
    incoming: &ResolvedClip,
    visual: bool,
    audio: bool,
    timebase: u32,
    sequence_duration: RationalTime,
) -> bool {
    let duration = value.record_window.duration;
    if !positive(duration, timebase)
        || !nonnegative(value.cut_time, timebase)
        || !nonnegative(value.record_window.start, timebase)
        || !nonnegative(value.outgoing_handle.offset, timebase)
        || !nonnegative(value.outgoing_handle.duration, timebase)
        || !nonnegative(value.incoming_handle.offset, timebase)
        || !nonnegative(value.incoming_handle.duration, timebase)
    {
        return false;
    }
    let outgoing_duration = match value.alignment {
        TransitionAlignment::BeforeCut => duration.value,
        TransitionAlignment::Centered => duration.value / 2,
        TransitionAlignment::AfterCut => 0,
    };
    let incoming_duration = duration.value - outgoing_duration;
    let expected_window = value.cut_time.value.checked_sub(outgoing_duration);
    let expected_offset = outgoing
        .record_range
        .duration
        .value
        .checked_sub(outgoing_duration);
    ((visual && outgoing.visual.is_some() && incoming.visual.is_some())
        || (audio && outgoing.audio.is_some() && incoming.audio.is_some()))
        && outgoing.record_range.end().ok() == Some(value.cut_time)
        && incoming.record_range.start == value.cut_time
        && expected_window == Some(value.record_window.start.value)
        && expected_offset == Some(value.outgoing_handle.offset.value)
        && value.outgoing_handle.duration.value == outgoing_duration
        && value.incoming_handle.offset.value == 0
        && value.incoming_handle.duration.value == incoming_duration
        && value.incoming_handle.duration <= incoming.record_range.duration
        && value
            .record_window
            .end()
            .is_ok_and(|end| end <= sequence_duration)
}

fn validate_window_overlap(check: &mut Check, transitions: &[ResolvedTransition]) {
    for pair in transitions.windows(2) {
        if pair[0].incoming_clip_id != pair[1].outgoing_clip_id {
            continue;
        }
        let overlaps = pair[0]
            .record_window
            .end()
            .is_ok_and(|end| end > pair[1].record_window.start);
        if overlaps {
            check.push(
                "PLAN_TRANSITION_WINDOW_OVERLAP",
                Some(pair[0].incoming_clip_id.to_string()),
                "incoming and outgoing transition windows overlap on the same clip",
            );
        }
    }
}

fn positive(value: RationalTime, timebase: u32) -> bool {
    value.is_valid() && value.timescale == timebase && value.value > 0
}

fn nonnegative(value: RationalTime, timebase: u32) -> bool {
    value.is_valid() && value.timescale == timebase && value.value >= 0
}

mod contract;

use std::collections::BTreeSet;

use veac_plan::canonical::BlendMode;
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
                && endpoints.is_some_and(|clips| clips.iter().any(incompatible_blend))
            {
                check.push(
                    "PLAN_TRANSITION_BLEND_UNSUPPORTED",
                    Some(transition.outgoing_clip_id.to_string()),
                    "visual transition endpoints must use normal blend mode",
                );
            }
            if track.state.visual_enabled
                && endpoints.is_some_and(|clips| endpoint_z(&clips[0]) != endpoint_z(&clips[1]))
            {
                check.push(
                    "PLAN_TRANSITION_Z_ORDER_UNSUPPORTED",
                    Some(transition.outgoing_clip_id.to_string()),
                    "visual transition endpoints must have equal z-index",
                );
            }
            if !pairs.insert(pair)
                || endpoints.is_none_or(|clips| {
                    !contract::valid(
                        transition,
                        &clips[0],
                        &clips[1],
                        &track.clips,
                        track.kind,
                        track.state.visual_enabled,
                        timebase,
                        sequence.duration,
                    )
                })
            {
                check.push(
                    "PLAN_TRANSITION_INVALID",
                    Some(transition.outgoing_clip_id.to_string()),
                    "transition topology or exact overlap ranges are invalid",
                );
            }
        }
        validate_window_overlap(check, &track.transitions);
    }
}

fn validate_window_overlap(check: &mut Check, transitions: &[ResolvedTransition]) {
    for pair in transitions.windows(2) {
        if pair[0].incoming_clip_id != pair[1].outgoing_clip_id {
            continue;
        }
        if contract::intersects(pair[0].record_window, pair[1].record_window) {
            check.push(
                "PLAN_TRANSITION_WINDOW_OVERLAP",
                Some(pair[0].incoming_clip_id.to_string()),
                "transition windows overlap on the shared middle clip",
            );
        }
    }
}

fn incompatible_blend(clip: &ResolvedClip) -> bool {
    clip.visual
        .as_ref()
        .is_some_and(|visual| visual.compositing.blend_mode != BlendMode::Normal)
}

fn endpoint_z(clip: &ResolvedClip) -> i32 {
    clip.visual
        .as_ref()
        .map_or(0, |visual| visual.compositing.z_index)
}

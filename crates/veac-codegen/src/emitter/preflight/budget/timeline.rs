use veac_plan::canonical::*;
use veac_plan::{ResolvedClip, ResolvedRenderPlan, ResolvedSourceTimeMap};

use super::super::Check;

pub(super) fn validate(check: &mut Check, plan: &ResolvedRenderPlan) {
    for sequence in &plan.sequences {
        if !duration_within_seconds(sequence.duration, MAX_TIMELINE_SECONDS) {
            check.push(
                "PLAN_BUDGET_TIMELINE_DURATION",
                Some(sequence.id.to_string()),
                "resolved timeline duration exceeds the untrusted execution budget",
            );
        }
        for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
            if reverse_exceeds(clip) {
                check.push(
                    "PLAN_BUDGET_REVERSE_DURATION",
                    Some(clip.id.to_string()),
                    "a reverse source interval exceeds the in-memory buffering budget",
                );
            }
        }
    }
}

fn reverse_exceeds(clip: &ResolvedClip) -> bool {
    let Some(mapping) = clip.source_mapping.as_ref() else {
        return false;
    };
    match &mapping.time_map {
        ResolvedSourceTimeMap::Linear {
            source_range_per_repeat,
            direction: PlaybackDirection::Reverse,
            ..
        } => !duration_within_seconds(
            source_range_per_repeat.duration,
            MAX_REVERSE_BUFFERED_SECONDS,
        ),
        ResolvedSourceTimeMap::Curve { segments } => segments.iter().any(|segment| {
            let span = segment
                .source_start
                .value
                .checked_sub(segment.source_end.value)
                .filter(|value| *value > 0);
            span.is_some_and(|value| {
                !duration_within_seconds(
                    RationalTime {
                        value,
                        timescale: segment.source_start.timescale,
                    },
                    MAX_REVERSE_BUFFERED_SECONDS,
                )
            })
        }),
        _ => false,
    }
}

#[cfg(test)]
#[path = "timeline/tests.rs"]
mod tests;

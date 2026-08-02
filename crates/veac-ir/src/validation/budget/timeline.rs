use crate::*;

use super::super::Validator;

pub(super) fn validate(validator: &mut Validator, project: &Project) {
    for sequence in &project.sequences {
        if sequence_duration(sequence)
            .is_some_and(|value| !duration_within_seconds(value, MAX_TIMELINE_SECONDS))
        {
            validator.push(
                "BUDGET_TIMELINE_DURATION",
                Some(sequence.id.to_string()),
                format!("/project/sequences/{}/tracks", sequence.id),
                "authored timeline duration exceeds the untrusted render execution budget",
                Some("split the timeline into shorter sequences"),
            );
        }
        for clip in sequence.tracks.iter().flat_map(|track| &track.clips) {
            if reverse_exceeds(clip) {
                validator.push(
                    "BUDGET_REVERSE_DURATION",
                    Some(clip.id.to_string()),
                    format!("/project/sequences/{}/tracks", sequence.id),
                    "a reverse source interval exceeds the in-memory buffering budget",
                    Some("split the reversed source interval into shorter clips"),
                );
            }
        }
    }
}

pub(super) fn sequence_duration(sequence: &Sequence) -> Option<RationalTime> {
    sequence
        .tracks
        .iter()
        .flat_map(|track| &track.clips)
        .filter_map(|clip| clip.record_range.end().ok())
        .max_by(|left, right| left.partial_cmp(right).unwrap_or(std::cmp::Ordering::Equal))
}

fn reverse_exceeds(clip: &Clip) -> bool {
    let Some(mapping) = &clip.source_mapping else {
        return false;
    };
    match &mapping.time_map {
        SourceTimeMap::Linear {
            rate,
            repeat,
            direction: PlaybackDirection::Reverse,
            ..
        } => !scaled_duration_within_seconds(
            clip.record_range.duration,
            *rate,
            *repeat,
            MAX_REVERSE_BUFFERED_SECONDS,
        ),
        SourceTimeMap::Curve { segments } => segments.iter().any(|segment| {
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
#[path = "timeline_tests.rs"]
mod tests;

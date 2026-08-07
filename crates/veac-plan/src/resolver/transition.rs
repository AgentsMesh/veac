use std::collections::BTreeSet;

use veac_ir::{
    Clip, RationalTime, RelationId, SequenceId, TimeRange, TrackId, TransitionAlignment,
};

use super::PlanResolver;
use crate::{EffectiveTrackState, ResolvedClip, ResolvedTransition};

impl PlanResolver<'_> {
    pub(super) fn resolve_transitions(
        &mut self,
        sequence_id: &SequenceId,
        track_id: &TrackId,
        resolved: &[ResolvedClip],
        state: EffectiveTrackState,
    ) -> Vec<ResolvedTransition> {
        if !state.visual_enabled && !state.audio_enabled {
            return Vec::new();
        }
        let included: BTreeSet<_> = resolved.iter().map(|clip| &clip.id).collect();
        self.relations
            .transitions(sequence_id, track_id)
            .into_iter()
            .filter(|edge| {
                included.contains(&edge.from.clip.id) && included.contains(&edge.to.clip.id)
            })
            .filter_map(|edge| {
                self.build_transition(
                    edge.relation_id,
                    edge.from.clip,
                    edge.to.clip,
                    edge.transition,
                )
            })
            .collect()
    }

    pub(super) fn build_transition(
        &mut self,
        relation_id: &RelationId,
        outgoing: &Clip,
        incoming: &Clip,
        transition: &veac_ir::Transition,
    ) -> Option<ResolvedTransition> {
        let scale = transition.duration.timescale;
        if outgoing.record_range.start.timescale != scale
            || outgoing.record_range.duration.timescale != scale
            || incoming.record_range.start.timescale != scale
            || incoming.record_range.duration.timescale != scale
            || transition.alignment != TransitionAlignment::Centered
        {
            self.push_internal(
                "TRANSITION_TIMEBASE",
                outgoing.id.to_string(),
                "validated transition timescales differ".to_owned(),
            );
            return None;
        }
        let outgoing_end = outgoing.record_range.end().ok()?;
        let duration = outgoing_end
            .value
            .checked_sub(incoming.record_range.start.value)?;
        if duration != transition.duration.value {
            self.push_internal(
                "TRANSITION_OVERLAP",
                outgoing.id.to_string(),
                "validated transition duration differs from endpoint overlap".to_owned(),
            );
            return None;
        }
        let record_window = TimeRange {
            start: incoming.record_range.start,
            duration: transition.duration,
        };
        let outgoing_start = incoming
            .record_range
            .start
            .value
            .checked_sub(outgoing.record_range.start.value)?;
        let local_duration = transition.duration;
        Some(ResolvedTransition {
            relation_id: relation_id.clone(),
            kind: transition.kind.clone(),
            outgoing_clip_id: outgoing.id.clone(),
            incoming_clip_id: incoming.id.clone(),
            alignment: transition.alignment,
            cut_time: RationalTime {
                value: record_window.start.value.checked_add(duration / 2)?,
                timescale: scale,
            },
            record_window,
            outgoing_range: TimeRange {
                start: RationalTime {
                    value: outgoing_start,
                    timescale: scale,
                },
                duration: local_duration,
            },
            incoming_range: TimeRange {
                start: RationalTime {
                    value: 0,
                    timescale: scale,
                },
                duration: local_duration,
            },
        })
    }
}

use std::collections::BTreeSet;

use veac_ir::{
    Clip, RationalTime, RelationId, SequenceId, TimeRange, TrackId, TransitionAlignment,
};

use super::PlanResolver;
use crate::{EffectiveTrackState, ResolvedClip, ResolvedTransition, TransitionHandle};

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
        if outgoing.record_range.duration.timescale != scale
            || incoming.record_range.duration.timescale != scale
            || incoming.record_range.start.timescale != scale
        {
            self.push_internal(
                "TRANSITION_TIMEBASE",
                outgoing.id.to_string(),
                "validated transition timescales differ".to_owned(),
            );
            return None;
        }
        let duration = transition.duration.value;
        let outgoing_duration = match transition.alignment {
            TransitionAlignment::BeforeCut => duration,
            TransitionAlignment::Centered => duration / 2,
            TransitionAlignment::AfterCut => 0,
        };
        let incoming_duration = duration - outgoing_duration;
        let cut = incoming.record_range.start;
        let window_start = RationalTime {
            value: cut.value.checked_sub(outgoing_duration)?,
            timescale: scale,
        };
        let outgoing_offset = RationalTime {
            value: outgoing
                .record_range
                .duration
                .value
                .checked_sub(outgoing_duration)?,
            timescale: scale,
        };
        Some(ResolvedTransition {
            relation_id: relation_id.clone(),
            kind: transition.kind.clone(),
            outgoing_clip_id: outgoing.id.clone(),
            incoming_clip_id: incoming.id.clone(),
            alignment: transition.alignment,
            cut_time: cut,
            record_window: TimeRange {
                start: window_start,
                duration: transition.duration,
            },
            outgoing_handle: TransitionHandle {
                offset: outgoing_offset,
                duration: RationalTime {
                    value: outgoing_duration,
                    timescale: scale,
                },
            },
            incoming_handle: TransitionHandle {
                offset: RationalTime {
                    value: 0,
                    timescale: scale,
                },
                duration: RationalTime {
                    value: incoming_duration,
                    timescale: scale,
                },
            },
        })
    }
}

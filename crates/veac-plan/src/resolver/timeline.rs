mod duration;
mod state;

use std::cmp::Ordering;

use veac_ir::{SequenceId, Track};

use super::PlanResolver;
use crate::{ResolutionDiagnostic, ResolutionErrorKind, ResolvedSequence, ResolvedTrack};

impl PlanResolver<'_> {
    pub(super) fn resolve_sequence(&mut self, sequence_id: &SequenceId) {
        if self.sequences.contains_key(sequence_id) {
            return;
        }
        if !self.visiting.insert(sequence_id.clone()) {
            self.push_internal(
                "SEQUENCE_CYCLE",
                sequence_id.to_string(),
                "validated sequence graph became cyclic".to_owned(),
            );
            return;
        }
        let Some(sequence) = self
            .envelope
            .project
            .sequences
            .iter()
            .find(|sequence| sequence.id == *sequence_id)
            .cloned()
        else {
            self.push_internal(
                "SEQUENCE_NOT_FOUND",
                sequence_id.to_string(),
                "validated sequence disappeared during resolution".to_owned(),
            );
            self.visiting.remove(sequence_id);
            return;
        };
        let duration = duration::authored(&sequence.tracks, self.envelope.project.timebase);
        if duration.value <= 0 {
            self.push(ResolutionDiagnostic::new(
                ResolutionErrorKind::TimelineDurationUnavailable,
                "SEQUENCE_DURATION_UNAVAILABLE",
                Some(sequence.id.to_string()),
                format!("/project/sequences/{}", sequence.id),
                "render resolution requires at least one clip to derive timeline duration",
            ));
        }
        let has_solo = sequence
            .tracks
            .iter()
            .any(|track| track.state.enabled && track.state.solo);
        let authored_applies = sequence.applies.clone();
        let mut authored_tracks: Vec<_> = sequence.tracks.into_iter().enumerate().collect();
        authored_tracks.sort_by(|left, right| {
            (left.1.order, left.0, &left.1.id).cmp(&(right.1.order, right.0, &right.1.id))
        });
        let mut tracks = Vec::with_capacity(authored_tracks.len());
        for (source_order, track) in authored_tracks {
            let Some(source_order) = self.source_order(source_order, track.id.to_string()) else {
                continue;
            };
            tracks.push(self.resolve_track(&sequence.id, track, source_order, has_solo));
        }
        let applies = self.resolve_applies(&sequence.id, &authored_applies, &tracks);
        let resolved = ResolvedSequence {
            id: sequence.id.clone(),
            name: sequence.name,
            settings: sequence.settings,
            duration,
            tracks,
            applies,
        };
        self.visiting.remove(sequence_id);
        self.sequence_order.push(sequence.id.clone());
        self.sequences.insert(sequence.id, resolved);
    }

    fn resolve_track(
        &mut self,
        sequence_id: &SequenceId,
        track: Track,
        source_order: u32,
        has_solo: bool,
    ) -> ResolvedTrack {
        let state = state::effective(&track, has_solo, self.config);
        let routing = state::routing(&track, state);
        let mut clips = Vec::new();
        if state.include_in_render {
            for (index, clip) in track.clips.iter().enumerate() {
                if !clip.enabled {
                    continue;
                }
                let Some(order) = self.source_order(index, clip.id.to_string()) else {
                    continue;
                };
                if let Some(resolved) =
                    self.resolve_clip(sequence_id, clip, track.kind, state, order)
                {
                    clips.push(resolved);
                }
            }
        }
        clips.sort_by(|left, right| {
            left.record_range
                .start
                .partial_cmp(&right.record_range.start)
                .unwrap_or(Ordering::Equal)
                .then_with(|| left.source_order.cmp(&right.source_order))
                .then_with(|| left.id.cmp(&right.id))
        });
        let transitions = self.resolve_transitions(sequence_id, &track.id, &clips, state);
        ResolvedTrack {
            id: track.id,
            source_order,
            kind: track.kind,
            order: track.order,
            placement_mode: track.placement_mode,
            state,
            routing,
            clips,
            transitions,
        }
    }

    pub(super) fn source_order(&mut self, index: usize, id: String) -> Option<u32> {
        match u32::try_from(index) {
            Ok(value) => Some(value),
            Err(error) => {
                self.push_internal("SOURCE_ORDER_OVERFLOW", id, error.to_string());
                None
            }
        }
    }
}

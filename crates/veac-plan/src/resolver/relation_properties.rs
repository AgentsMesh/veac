use veac_ir::{ApplyId, ItemId, RelationSignal, SequenceId, SidechainSource};

use super::PlanResolver;
use crate::{ResolvedMatte, ResolvedSidechain};

impl PlanResolver<'_> {
    pub(super) fn resolved_apply_matte(
        &self,
        sequence_id: &SequenceId,
        apply_id: &ApplyId,
    ) -> Option<ResolvedMatte> {
        let edge = self.relations.matte_for_apply(sequence_id, apply_id)?;
        Some(ResolvedMatte {
            relation_id: edge.relation_id.clone(),
            source_clip_id: edge.producer.clip.id.clone(),
            mode: edge.parameters.mode,
            invert: edge.parameters.invert,
        })
    }

    pub(super) fn resolved_matte(
        &self,
        sequence_id: &SequenceId,
        item_id: &ItemId,
    ) -> Option<ResolvedMatte> {
        let edge = self.relations.matte_for_consumer(sequence_id, item_id)?;
        Some(ResolvedMatte {
            relation_id: edge.relation_id.clone(),
            source_clip_id: edge.producer.clip.id.clone(),
            mode: edge.parameters.mode,
            invert: edge.parameters.invert,
        })
    }

    pub(super) fn resolved_sidechain(
        &self,
        sequence_id: &SequenceId,
        item_id: &ItemId,
    ) -> Option<ResolvedSidechain> {
        let edge = self.relations.sidechain_for_target(sequence_id, item_id)?;
        let source = match edge.key {
            RelationSignal::Track(track) => SidechainSource::Track {
                track_id: track.id.clone(),
            },
            RelationSignal::Bus { bus_id, .. } => SidechainSource::Bus {
                bus_id: bus_id.clone(),
            },
        };
        Some(ResolvedSidechain {
            relation_id: edge.relation_id.clone(),
            source,
            threshold_db: edge.parameters.threshold_db,
            ratio: edge.parameters.ratio,
            attack_ms: edge.parameters.attack_ms,
            release_ms: edge.parameters.release_ms,
            active_range: edge.parameters.active_range,
        })
    }
}

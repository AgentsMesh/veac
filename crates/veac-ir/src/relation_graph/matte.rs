use crate::*;

use super::{RelationApply, RelationGraph, RelationItem};

#[derive(Clone, Copy, Debug)]
pub struct MatteEdge<'a> {
    pub relation_id: &'a RelationId,
    pub sequence_id: &'a SequenceId,
    pub producer: RelationItem<'a>,
    pub consumer: RelationItem<'a>,
    pub parameters: &'a MatteRelationParameters,
}

#[derive(Clone, Copy, Debug)]
pub struct ApplyMatteEdge<'a> {
    pub relation_id: &'a RelationId,
    pub sequence_id: &'a SequenceId,
    pub producer: RelationItem<'a>,
    pub consumer: RelationApply<'a>,
    pub parameters: &'a MatteRelationParameters,
}

impl<'a> RelationGraph<'a> {
    pub fn matte(&self, relation: &'a Relation) -> Option<MatteEdge<'a>> {
        let RelationKind::Matte {
            producer,
            consumer,
            parameters,
        } = &relation.kind
        else {
            return None;
        };
        Some(MatteEdge {
            relation_id: &relation.id,
            sequence_id: &relation.sequence_id,
            producer: self.item(&relation.sequence_id, producer.item_id()?)?,
            consumer: self.item(&relation.sequence_id, consumer.item_id()?)?,
            parameters,
        })
    }

    pub fn mattes(&self, sequence_id: &SequenceId) -> Vec<MatteEdge<'a>> {
        let mut edges: Vec<_> = self
            .relations(sequence_id)
            .into_iter()
            .filter_map(|relation| self.matte(relation))
            .collect();
        edges.sort_by_key(|edge| {
            (
                track_position(edge.consumer),
                edge.consumer.position,
                edge.relation_id.as_str(),
            )
        });
        edges
    }

    pub fn apply_matte(&self, relation: &'a Relation) -> Option<ApplyMatteEdge<'a>> {
        let RelationKind::Matte {
            producer,
            consumer,
            parameters,
        } = &relation.kind
        else {
            return None;
        };
        Some(ApplyMatteEdge {
            relation_id: &relation.id,
            sequence_id: &relation.sequence_id,
            producer: self.item(&relation.sequence_id, producer.item_id()?)?,
            consumer: self.apply(&relation.sequence_id, consumer.apply_id()?)?,
            parameters,
        })
    }

    pub fn apply_mattes(&self, sequence_id: &SequenceId) -> Vec<ApplyMatteEdge<'a>> {
        let mut edges: Vec<_> = self
            .relations(sequence_id)
            .into_iter()
            .filter_map(|relation| self.apply_matte(relation))
            .collect();
        edges.sort_by_key(|edge| (edge.consumer.position, edge.relation_id.as_str()));
        edges
    }

    pub fn matte_for_apply(
        &self,
        sequence_id: &SequenceId,
        apply_id: &ApplyId,
    ) -> Option<ApplyMatteEdge<'a>> {
        self.apply_mattes(sequence_id)
            .into_iter()
            .find(|edge| edge.consumer.apply.id == *apply_id)
    }

    pub fn matte_for_consumer(
        &self,
        sequence_id: &SequenceId,
        item_id: &ItemId,
    ) -> Option<MatteEdge<'a>> {
        self.mattes(sequence_id)
            .into_iter()
            .find(|edge| edge.consumer.clip.id == *item_id)
    }
}

fn track_position(item: RelationItem<'_>) -> usize {
    item.sequence
        .tracks
        .iter()
        .position(|track| track.id == item.track.id)
        .unwrap_or(usize::MAX)
}

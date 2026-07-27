use std::collections::BTreeMap;

use crate::{
    Apply, ApplyId, Clip, ItemId, Project, Relation, RelationEndpoint, RelationId, RelationKind,
    Sequence, SequenceId, Track, TrackId, Transition,
};

mod matte;
mod membership;
mod scope;
mod sidechain;

pub use matte::{ApplyMatteEdge, MatteEdge};
pub use membership::{AvLinkEdge, GroupEdge};
pub use scope::RelationSequence;
pub use sidechain::{RelationSignal, SidechainEdge};

#[derive(Clone, Copy, Debug)]
pub struct RelationItem<'a> {
    pub sequence: &'a Sequence,
    pub track: &'a Track,
    pub clip: &'a Clip,
    pub position: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct RelationApply<'a> {
    pub sequence: &'a Sequence,
    pub apply: &'a Apply,
    pub position: usize,
}

#[derive(Clone, Copy)]
pub struct TransitionEdge<'a> {
    pub relation_id: &'a RelationId,
    pub sequence_id: &'a SequenceId,
    pub from: RelationItem<'a>,
    pub to: RelationItem<'a>,
    pub transition: &'a Transition,
}

pub struct RelationGraph<'a> {
    relations: &'a [Relation],
    sequences: BTreeMap<&'a str, RelationSequence<'a>>,
}

impl<'a> RelationGraph<'a> {
    pub fn project(project: &'a Project) -> Self {
        let sequences = project
            .sequences
            .iter()
            .map(|sequence| (sequence.id.as_str(), RelationSequence::new(sequence)))
            .collect();
        Self {
            relations: &project.relations,
            sequences,
        }
    }

    pub fn scoped(sequence: &'a Sequence, relations: &'a [Relation]) -> Self {
        Self {
            relations,
            sequences: [(sequence.id.as_str(), RelationSequence::new(sequence))]
                .into_iter()
                .collect(),
        }
    }

    pub fn sequence(&self, id: &SequenceId) -> Option<&'a Sequence> {
        self.sequences.get(id.as_str()).map(|index| index.sequence)
    }

    pub fn scope(&self, id: &SequenceId) -> Option<&RelationSequence<'a>> {
        self.sequences.get(id.as_str())
    }

    pub fn item(&self, sequence_id: &SequenceId, item_id: &ItemId) -> Option<RelationItem<'a>> {
        self.sequences
            .get(sequence_id.as_str())?
            .items
            .get(item_id.as_str())
            .copied()
    }

    pub fn apply(&self, sequence_id: &SequenceId, apply_id: &ApplyId) -> Option<RelationApply<'a>> {
        let scope = self.sequences.get(sequence_id.as_str())?;
        let (apply, position) = scope.apply(apply_id)?;
        Some(RelationApply {
            sequence: scope.sequence(),
            apply,
            position,
        })
    }

    pub fn contains_endpoint(&self, sequence_id: &SequenceId, endpoint: &RelationEndpoint) -> bool {
        self.sequences
            .get(sequence_id.as_str())
            .is_some_and(|index| index.contains(endpoint))
    }

    pub fn relations(&self, sequence_id: &SequenceId) -> Vec<&'a Relation> {
        self.relations
            .iter()
            .filter(|relation| relation.sequence_id == *sequence_id)
            .collect()
    }

    pub fn transition(&self, relation: &'a Relation) -> Option<TransitionEdge<'a>> {
        let RelationKind::Transition {
            from,
            to,
            transition,
        } = &relation.kind
        else {
            return None;
        };
        let (
            RelationEndpoint::Item {
                item_id: from_item_id,
            },
            RelationEndpoint::Item {
                item_id: to_item_id,
            },
        ) = (from, to)
        else {
            return None;
        };
        Some(TransitionEdge {
            relation_id: &relation.id,
            sequence_id: &relation.sequence_id,
            from: self.item(&relation.sequence_id, from_item_id)?,
            to: self.item(&relation.sequence_id, to_item_id)?,
            transition,
        })
    }

    pub fn transitions(
        &self,
        sequence_id: &SequenceId,
        track_id: &TrackId,
    ) -> Vec<TransitionEdge<'a>> {
        let mut edges: Vec<_> = self
            .relations(sequence_id)
            .into_iter()
            .filter_map(|relation| self.transition(relation))
            .filter(|edge| edge.from.track.id == *track_id)
            .collect();
        edges.sort_by(|left, right| {
            left.from
                .position
                .cmp(&right.from.position)
                .then_with(|| left.relation_id.cmp(right.relation_id))
        });
        edges
    }

    pub fn transition_from(
        &self,
        sequence_id: &SequenceId,
        item_id: &ItemId,
    ) -> Option<TransitionEdge<'a>> {
        self.relations(sequence_id)
            .into_iter()
            .filter_map(|relation| self.transition(relation))
            .find(|edge| edge.from.clip.id == *item_id)
    }
}

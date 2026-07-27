use crate::*;

use super::{RelationGraph, RelationItem};

#[derive(Clone, Debug)]
pub enum RelationSignal<'a> {
    Track(&'a Track),
    Bus {
        bus_id: &'a BusId,
        tracks: Vec<&'a Track>,
    },
}

#[derive(Clone, Debug)]
pub struct SidechainEdge<'a> {
    pub relation_id: &'a RelationId,
    pub sequence_id: &'a SequenceId,
    pub key: RelationSignal<'a>,
    pub target: RelationItem<'a>,
    pub parameters: &'a SidechainRelationParameters,
}

impl<'a> RelationGraph<'a> {
    pub fn sidechain(&self, relation: &'a Relation) -> Option<SidechainEdge<'a>> {
        let RelationKind::Sidechain {
            key,
            target,
            parameters,
        } = &relation.kind
        else {
            return None;
        };
        let scope = self.scope(&relation.sequence_id)?;
        if !scope.contains(key) {
            return None;
        }
        Some(SidechainEdge {
            relation_id: &relation.id,
            sequence_id: &relation.sequence_id,
            key: match key {
                RelationEndpoint::Track { track_id } => {
                    RelationSignal::Track(scope.track(track_id)?)
                }
                RelationEndpoint::Bus { bus_id } => RelationSignal::Bus {
                    bus_id,
                    tracks: scope.bus_tracks(bus_id),
                },
                RelationEndpoint::Item { .. } | RelationEndpoint::Apply { .. } => return None,
            },
            target: self.item(&relation.sequence_id, target.item_id()?)?,
            parameters,
        })
    }

    pub fn sidechains(&self, sequence_id: &SequenceId) -> Vec<SidechainEdge<'a>> {
        let mut edges: Vec<_> = self
            .relations(sequence_id)
            .into_iter()
            .filter_map(|relation| self.sidechain(relation))
            .collect();
        edges.sort_by_key(|edge| {
            (
                track_position(edge.target),
                edge.target.position,
                edge.relation_id.as_str(),
            )
        });
        edges
    }

    pub fn sidechain_for_target(
        &self,
        sequence_id: &SequenceId,
        item_id: &ItemId,
    ) -> Option<SidechainEdge<'a>> {
        self.sidechains(sequence_id)
            .into_iter()
            .find(|edge| edge.target.clip.id == *item_id)
    }
}

fn track_position(item: RelationItem<'_>) -> usize {
    item.sequence
        .tracks
        .iter()
        .position(|track| track.id == item.track.id)
        .unwrap_or(usize::MAX)
}

use std::collections::BTreeSet;

use crate::{ItemId, Relation, RelationEndpoint, RelationId, RelationKind, SequenceId};

use super::{RelationGraph, RelationItem};

#[derive(Clone, Debug)]
pub struct GroupEdge<'a> {
    pub relation_id: &'a RelationId,
    pub sequence_id: &'a SequenceId,
    pub members: Vec<RelationItem<'a>>,
}

#[derive(Clone, Debug)]
pub struct AvLinkEdge<'a> {
    pub relation_id: &'a RelationId,
    pub sequence_id: &'a SequenceId,
    pub video: RelationItem<'a>,
    pub audio: Vec<RelationItem<'a>>,
}

impl<'a> RelationGraph<'a> {
    pub fn group(&self, relation: &'a Relation) -> Option<GroupEdge<'a>> {
        let RelationKind::Group { members } = &relation.kind else {
            return None;
        };
        Some(GroupEdge {
            relation_id: &relation.id,
            sequence_id: &relation.sequence_id,
            members: self.items(&relation.sequence_id, members)?,
        })
    }

    pub fn groups(&self, sequence_id: &SequenceId) -> Vec<GroupEdge<'a>> {
        let mut edges: Vec<_> = self
            .relations(sequence_id)
            .into_iter()
            .filter_map(|relation| self.group(relation))
            .collect();
        edges.sort_by(|left, right| left.relation_id.cmp(right.relation_id));
        edges
    }

    pub fn group_for_item(
        &self,
        sequence_id: &SequenceId,
        item_id: &ItemId,
    ) -> Option<GroupEdge<'a>> {
        self.groups(sequence_id)
            .into_iter()
            .find(|edge| edge.members.iter().any(|item| item.clip.id == *item_id))
    }

    pub fn av_link(&self, relation: &'a Relation) -> Option<AvLinkEdge<'a>> {
        let RelationKind::AvLink { video, audio } = &relation.kind else {
            return None;
        };
        let RelationEndpoint::Item { item_id } = video else {
            return None;
        };
        Some(AvLinkEdge {
            relation_id: &relation.id,
            sequence_id: &relation.sequence_id,
            video: self.item(&relation.sequence_id, item_id)?,
            audio: self.items(&relation.sequence_id, audio)?,
        })
    }

    pub fn av_links(&self, sequence_id: &SequenceId) -> Vec<AvLinkEdge<'a>> {
        let mut edges: Vec<_> = self
            .relations(sequence_id)
            .into_iter()
            .filter_map(|relation| self.av_link(relation))
            .collect();
        edges.sort_by(|left, right| left.relation_id.cmp(right.relation_id));
        edges
    }

    pub fn av_link_for_item(
        &self,
        sequence_id: &SequenceId,
        item_id: &ItemId,
    ) -> Option<AvLinkEdge<'a>> {
        self.av_links(sequence_id).into_iter().find(|edge| {
            edge.video.clip.id == *item_id || edge.audio.iter().any(|item| item.clip.id == *item_id)
        })
    }

    pub fn membership_component(
        &self,
        sequence_id: &SequenceId,
        item_id: &ItemId,
    ) -> Option<Vec<RelationItem<'a>>> {
        self.item(sequence_id, item_id)?;
        let groups = self.groups(sequence_id);
        let links = self.av_links(sequence_id);
        let mut members = BTreeSet::from([item_id.clone()]);
        loop {
            let previous = members.len();
            for edge in &groups {
                extend_if_connected(&mut members, edge.members.iter().map(|item| &item.clip.id));
            }
            for edge in &links {
                extend_if_connected(
                    &mut members,
                    std::iter::once(&edge.video.clip.id)
                        .chain(edge.audio.iter().map(|item| &item.clip.id)),
                );
            }
            if members.len() == previous {
                break;
            }
        }
        Some(
            members
                .into_iter()
                .map(|id| self.item(sequence_id, &id).expect("resolved member"))
                .collect(),
        )
    }

    fn items(
        &self,
        sequence_id: &SequenceId,
        endpoints: &[RelationEndpoint],
    ) -> Option<Vec<RelationItem<'a>>> {
        endpoints
            .iter()
            .map(|endpoint| match endpoint {
                RelationEndpoint::Item { item_id } => self.item(sequence_id, item_id),
                RelationEndpoint::Apply { .. }
                | RelationEndpoint::Track { .. }
                | RelationEndpoint::Bus { .. } => None,
            })
            .collect()
    }
}

fn extend_if_connected<'a>(members: &mut BTreeSet<ItemId>, edge: impl Iterator<Item = &'a ItemId>) {
    let edge: Vec<_> = edge.cloned().collect();
    if edge.iter().any(|item| members.contains(item)) {
        members.extend(edge);
    }
}

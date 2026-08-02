use std::collections::BTreeSet;

use crate::edit::operations::relation_refs::{self, ItemTopology};
use crate::edit::{ensure_clip_unlocked, operation_error, ChangeSet, MarkChanged};
use crate::{
    Diagnostic, ItemId, Project, RelationEndpoint, RelationGraph, RelationId, RelationKind,
    SequenceId, TrackId,
};

#[cfg(test)]
mod tests;

pub(super) fn connected(
    project: &Project,
    clip_id: &ItemId,
) -> Result<(SequenceId, Vec<ItemId>), Diagnostic> {
    let sequence_id = sequence_id(project, clip_id)?;
    let graph = RelationGraph::project(project);
    let members = graph
        .membership_component(&sequence_id, clip_id)
        .ok_or_else(|| operation_error("/edit", "clip does not exist"))?;
    Ok((
        sequence_id,
        members
            .into_iter()
            .map(|item| item.clip.id.clone())
            .collect(),
    ))
}

pub(super) fn linked(project: &Project, clip_id: &ItemId) -> Result<Vec<ItemId>, Diagnostic> {
    let sequence_id = sequence_id(project, clip_id)?;
    let graph = RelationGraph::project(project);
    let Some(link) = graph.av_link_for_item(&sequence_id, clip_id) else {
        return Ok(vec![clip_id.clone()]);
    };
    let mut members = BTreeSet::from([link.video.clip.id.clone()]);
    members.extend(link.audio.iter().map(|item| item.clip.id.clone()));
    Ok(members.into_iter().collect())
}

pub(super) fn ensure_independent(project: &Project, clip_id: &ItemId) -> Result<(), Diagnostic> {
    let sequence_id = sequence_id(project, clip_id)?;
    let graph = RelationGraph::project(project);
    if graph.group_for_item(&sequence_id, clip_id).is_some()
        || graph.av_link_for_item(&sequence_id, clip_id).is_some()
    {
        return Err(operation_error(
            "/edit",
            "operation requires an independent clip; unlink or ungroup it first",
        ));
    }
    Ok(())
}

pub(super) fn ensure_unlocked(project: &Project, clip_ids: &[ItemId]) -> Result<(), Diagnostic> {
    for clip_id in clip_ids {
        ensure_clip_unlocked(project, clip_id)?;
    }
    Ok(())
}

pub(super) fn sequence_id(project: &Project, clip_id: &ItemId) -> Result<SequenceId, Diagnostic> {
    project
        .sequences
        .iter()
        .find(|sequence| {
            sequence
                .tracks
                .iter()
                .any(|track| track.clips.iter().any(|clip| clip.id == *clip_id))
        })
        .map(|sequence| sequence.id.clone())
        .ok_or_else(|| operation_error("/edit", "clip does not exist"))
}

pub(super) fn track_id(project: &Project, clip_id: &ItemId) -> Result<TrackId, Diagnostic> {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .find(|track| track.clips.iter().any(|clip| clip.id == *clip_id))
        .map(|track| track.id.clone())
        .ok_or_else(|| operation_error("/edit", "clip does not exist"))
}

pub(super) fn relationship_ids(
    project: &Project,
    sequence_id: &SequenceId,
    clip_id: &ItemId,
) -> (BTreeSet<RelationId>, BTreeSet<RelationId>) {
    let graph = RelationGraph::project(project);
    let groups = graph
        .group_for_item(sequence_id, clip_id)
        .map(|edge| edge.relation_id.clone())
        .into_iter()
        .collect();
    let links = graph
        .av_link_for_item(sequence_id, clip_id)
        .map(|edge| edge.relation_id.clone())
        .into_iter()
        .collect();
    (groups, links)
}

pub(super) fn detach_removed(
    project: &mut Project,
    sequence_id: &SequenceId,
    removed: &BTreeSet<ItemId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let membership_relations = project
        .relations
        .iter()
        .filter(|relation| &relation.sequence_id == sequence_id)
        .filter(|relation| match &relation.kind {
            RelationKind::Group { members } => members
                .iter()
                .any(|endpoint| removed_endpoint(endpoint, removed)),
            RelationKind::AvLink { video, audio } => {
                removed_endpoint(video, removed)
                    || audio
                        .iter()
                        .any(|endpoint| removed_endpoint(endpoint, removed))
            }
            _ => false,
        })
        .cloned()
        .collect::<Vec<_>>();
    for relation in &membership_relations {
        relation_refs::ensure_relation_unlocked_except(
            project,
            relation,
            removed,
            &BTreeSet::new(),
        )?;
    }
    let topology = removed
        .iter()
        .cloned()
        .map(|item_id| ItemTopology::Removed { item_id })
        .collect::<Vec<_>>();
    relation_refs::rewrite_item_relations(project, sequence_id, &topology, changed)?;
    let mut affected = BTreeSet::new();
    let mut discarded = BTreeSet::new();
    for relation in &mut project.relations {
        if &relation.sequence_id != sequence_id {
            continue;
        }
        match &mut relation.kind {
            RelationKind::Group { members } => {
                let before = members.len();
                members.retain(|endpoint| !removed_endpoint(endpoint, removed));
                if members.len() != before {
                    affected.insert(relation.id.clone());
                }
                if members.len() < 2 {
                    discarded.insert(relation.id.clone());
                }
            }
            RelationKind::AvLink { video, audio } => {
                if removed_endpoint(video, removed)
                    || audio
                        .iter()
                        .any(|endpoint| removed_endpoint(endpoint, removed))
                {
                    discarded.insert(relation.id.clone());
                }
            }
            _ => {}
        }
    }
    project
        .relations
        .retain(|relation| !discarded.contains(&relation.id));
    affected.extend(discarded);
    for relation_id in affected {
        changed.relation(relation_id);
        changed.project(project.id.clone());
    }
    Ok(())
}

fn removed_endpoint(endpoint: &RelationEndpoint, removed: &BTreeSet<ItemId>) -> bool {
    matches!(endpoint, RelationEndpoint::Item { item_id } if removed.contains(item_id))
}

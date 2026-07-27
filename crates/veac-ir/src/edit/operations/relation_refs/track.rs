use std::collections::BTreeSet;

use crate::edit::{ChangeSet, MarkChanged};
use crate::*;

#[cfg(test)]
#[path = "track/tests.rs"]
mod tests;

use super::lock::ensure_relation_unlocked_except;

pub(crate) fn remove_sequence(
    project: &mut Project,
    sequence_id: &SequenceId,
    changed: &mut ChangeSet,
) {
    let ids: Vec<_> = project
        .relations
        .iter()
        .filter(|item| item.sequence_id == *sequence_id)
        .map(|item| item.id.clone())
        .collect();
    project
        .relations
        .retain(|item| item.sequence_id != *sequence_id);
    mark(changed, &project.id, ids);
}

pub(crate) fn remove_track(
    project: &mut Project,
    sequence_id: &SequenceId,
    track_id: &TrackId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let sequence = project
        .sequences
        .iter()
        .find(|item| item.id == *sequence_id)
        .expect("validated sequence");
    let track = sequence
        .tracks
        .iter()
        .find(|item| item.id == *track_id)
        .expect("validated track");
    let items: BTreeSet<_> = track.clips.iter().map(|item| item.id.clone()).collect();
    let disappearing_bus = match &track.routing {
        TrackRouting::AudioBus { bus_id }
            if !sequence.tracks.iter().any(|item| {
                item.id != *track_id
                    && matches!(&item.routing, TrackRouting::AudioBus { bus_id: id } if id == bus_id)
            }) => Some(bus_id.clone()),
        _ => None,
    };
    let affected: Vec<_> = project
        .relations
        .iter()
        .filter(|relation| relation.sequence_id == *sequence_id)
        .filter(|relation| {
            relation_uses(&relation.kind, &items, track_id, disappearing_bus.as_ref())
        })
        .cloned()
        .collect();
    let skipped_tracks = BTreeSet::from([track_id.clone()]);
    for relation in &affected {
        ensure_relation_unlocked_except(project, relation, &items, &skipped_tracks)?;
    }
    let ids: BTreeSet<_> = affected.iter().map(|item| item.id.clone()).collect();
    project.relations.retain(|item| !ids.contains(&item.id));
    mark(changed, &project.id, ids);
    Ok(())
}

pub(crate) fn prune_disappeared_bus(
    project: &mut Project,
    sequence_id: &SequenceId,
    bus_id: &BusId,
    changed_track_id: &TrackId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let sequence = project
        .sequences
        .iter()
        .find(|item| item.id == *sequence_id)
        .expect("validated sequence");
    if sequence
        .tracks
        .iter()
        .any(|item| matches!(&item.routing, TrackRouting::AudioBus { bus_id: id } if id == bus_id))
    {
        return Ok(());
    }
    let affected: Vec<_> = project
        .relations
        .iter()
        .filter(|relation| {
            relation.sequence_id == *sequence_id
                && matches!(&relation.kind, RelationKind::Sidechain {
                    key: RelationEndpoint::Bus { bus_id: id }, ..
                } if id == bus_id)
        })
        .cloned()
        .collect();
    let skipped = BTreeSet::from([changed_track_id.clone()]);
    for relation in &affected {
        ensure_relation_unlocked_except(project, relation, &BTreeSet::new(), &skipped)?;
    }
    let ids: BTreeSet<_> = affected.iter().map(|item| item.id.clone()).collect();
    project.relations.retain(|item| !ids.contains(&item.id));
    mark(changed, &project.id, ids);
    Ok(())
}

fn relation_uses(
    kind: &RelationKind,
    items: &BTreeSet<ItemId>,
    track_id: &TrackId,
    bus_id: Option<&BusId>,
) -> bool {
    let item =
        |endpoint: &RelationEndpoint| endpoint.item_id().is_some_and(|id| items.contains(id));
    match kind {
        RelationKind::Transition { from, to, .. } => item(from) || item(to),
        RelationKind::Matte {
            producer, consumer, ..
        } => item(producer) || item(consumer),
        RelationKind::Sidechain { key, target, .. } => {
            item(target)
                || matches!(key, RelationEndpoint::Track { track_id: id } if id == track_id)
                || matches!((key, bus_id), (RelationEndpoint::Bus { bus_id: id }, Some(bus)) if id == bus)
        }
        RelationKind::Group { members } => members.iter().any(item),
        RelationKind::AvLink { video, audio } => item(video) || audio.iter().any(item),
    }
}

fn mark(
    changed: &mut ChangeSet,
    project_id: &ProjectId,
    ids: impl IntoIterator<Item = RelationId>,
) {
    let mut any = false;
    for id in ids {
        any = true;
        changed.relation(id);
    }
    if any {
        changed.project(project_id.clone());
    }
}

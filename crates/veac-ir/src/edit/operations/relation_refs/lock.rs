use std::collections::BTreeSet;

use crate::edit::operation_error;
use crate::*;

#[cfg(test)]
#[path = "lock/tests.rs"]
mod tests;

use super::super::apply_refs;

pub(crate) fn ensure_relation_unlocked(
    project: &Project,
    relation: &Relation,
) -> Result<(), Diagnostic> {
    ensure_relation_unlocked_except(project, relation, &BTreeSet::new(), &BTreeSet::new())
}

pub(crate) fn ensure_relation_unlocked_except(
    project: &Project,
    relation: &Relation,
    skipped_items: &BTreeSet<ItemId>,
    skipped_tracks: &BTreeSet<TrackId>,
) -> Result<(), Diagnostic> {
    let sequence = project
        .sequences
        .iter()
        .find(|item| item.id == relation.sequence_id)
        .ok_or_else(|| operation_error(relation.id.as_str(), "relation sequence does not exist"))?;
    for endpoint in endpoints(&relation.kind) {
        match endpoint {
            RelationEndpoint::Item { item_id } if !skipped_items.contains(item_id) => {
                let track = sequence
                    .tracks
                    .iter()
                    .find(|track| track.clips.iter().any(|clip| clip.id == *item_id))
                    .ok_or_else(|| {
                        operation_error(item_id.as_str(), "relation item does not exist")
                    })?;
                ensure_track(track, skipped_tracks)?;
            }
            RelationEndpoint::Track { track_id } if !skipped_tracks.contains(track_id) => {
                let track = sequence
                    .tracks
                    .iter()
                    .find(|track| track.id == *track_id)
                    .ok_or_else(|| {
                        operation_error(track_id.as_str(), "relation track does not exist")
                    })?;
                ensure_track(track, skipped_tracks)?;
            }
            RelationEndpoint::Apply { apply_id } => {
                let apply = sequence
                    .applies
                    .iter()
                    .find(|apply| apply.id == *apply_id)
                    .ok_or_else(|| {
                        operation_error(apply_id.as_str(), "relation apply does not exist")
                    })?;
                apply_refs::ensure_target_unlocked(sequence, &apply.target, skipped_tracks)?;
            }
            RelationEndpoint::Bus { bus_id } => {
                for track in &sequence.tracks {
                    if matches!(&track.routing, TrackRouting::AudioBus { bus_id: id } if id == bus_id)
                    {
                        ensure_track(track, skipped_tracks)?;
                    }
                }
            }
            _ => {}
        }
    }
    Ok(())
}

fn ensure_track(track: &Track, skipped: &BTreeSet<TrackId>) -> Result<(), Diagnostic> {
    if track.state.locked && !skipped.contains(&track.id) {
        Err(operation_error(
            track.id.as_str(),
            "relation endpoint track is locked",
        ))
    } else {
        Ok(())
    }
}

fn endpoints(kind: &RelationKind) -> Vec<&RelationEndpoint> {
    match kind {
        RelationKind::Transition { from, to, .. } => vec![from, to],
        RelationKind::Matte {
            producer, consumer, ..
        } => vec![producer, consumer],
        RelationKind::Sidechain { key, target, .. } => vec![key, target],
        RelationKind::Group { members } => members.iter().collect(),
        RelationKind::AvLink { video, audio } => {
            std::iter::once(video).chain(audio.iter()).collect()
        }
    }
}

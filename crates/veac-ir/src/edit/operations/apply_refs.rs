use std::collections::BTreeSet;

use crate::edit::operation_error;
use crate::*;

#[cfg(test)]
#[path = "apply_refs/tests.rs"]
mod tests;

pub(super) fn ensure_target_unlocked(
    sequence: &Sequence,
    target: &ApplyTarget,
    skipped: &BTreeSet<TrackId>,
) -> Result<(), Diagnostic> {
    for id in target_track_ids(sequence, target)? {
        let track = sequence
            .tracks
            .iter()
            .find(|track| track.id == id)
            .expect("target resolver returned existing track");
        if track.state.locked && !skipped.contains(&id) {
            return Err(operation_error(id.as_str(), "apply target track is locked"));
        }
    }
    Ok(())
}

pub(super) fn ensure_track_removable(
    project: &Project,
    track_id: &TrackId,
) -> Result<(), Diagnostic> {
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| sequence.tracks.iter().any(|track| track.id == *track_id))
        .ok_or_else(|| operation_error(track_id.as_str(), "track does not exist"))?;
    for apply in &sequence.applies {
        if target_track_ids(sequence, &apply.target)?.contains(track_id) {
            return Err(operation_error(
                track_id.as_str(),
                "track is referenced by an apply target",
            ));
        }
    }
    Ok(())
}

pub(super) fn ensure_items_removable(
    sequence: &Sequence,
    removed: &BTreeSet<ItemId>,
) -> Result<(), Diagnostic> {
    for apply in &sequence.applies {
        let ApplyTarget::ItemSet { item_ids } = &apply.target else {
            continue;
        };
        if let Some(id) = item_ids.iter().find(|id| removed.contains(*id)) {
            return Err(operation_error(
                id.as_str(),
                "item is referenced by an apply target",
            ));
        }
    }
    Ok(())
}

pub(super) fn ensure_order_change_unlocked(
    project: &Project,
    track_id: &TrackId,
    new_order: i32,
) -> Result<(), Diagnostic> {
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| sequence.tracks.iter().any(|track| track.id == *track_id))
        .ok_or_else(|| operation_error(track_id.as_str(), "track does not exist"))?;
    let mut changed = sequence.clone();
    changed
        .tracks
        .iter_mut()
        .find(|track| track.id == *track_id)
        .expect("located sequence owns track")
        .order = new_order;
    for apply in &sequence.applies {
        if !matches!(apply.target, ApplyTarget::CompositeBand { .. }) {
            continue;
        }
        let before = target_track_ids(sequence, &apply.target)?;
        let after = target_track_ids(&changed, &apply.target)?;
        if before != after {
            let all: BTreeSet<_> = before.union(&after).cloned().collect();
            for id in all {
                let locked = sequence
                    .tracks
                    .iter()
                    .find(|track| track.id == id)
                    .is_some_and(|track| track.state.locked);
                if locked {
                    return Err(operation_error(id.as_str(), "apply target track is locked"));
                }
            }
        }
    }
    Ok(())
}

pub(super) fn target_track_ids(
    sequence: &Sequence,
    target: &ApplyTarget,
) -> Result<BTreeSet<TrackId>, Diagnostic> {
    match target {
        ApplyTarget::Layer { track_id } => {
            require_track(sequence, track_id)?;
            Ok(BTreeSet::from([track_id.clone()]))
        }
        ApplyTarget::ItemSet { item_ids } => item_ids
            .iter()
            .map(|id| item_track(sequence, id).map(|track| track.id.clone()))
            .collect(),
        ApplyTarget::CompositeBand {
            from_track_id,
            through_track_id,
        } => {
            let from = require_track(sequence, from_track_id)?.order;
            let through = require_track(sequence, through_track_id)?.order;
            Ok(sequence
                .tracks
                .iter()
                .filter(|track| {
                    (from..=through).contains(&track.order)
                        && matches!(
                            track.kind,
                            TrackKind::Video | TrackKind::Visual | TrackKind::Caption
                        )
                })
                .map(|track| track.id.clone())
                .collect())
        }
    }
}

fn require_track<'a>(sequence: &'a Sequence, id: &TrackId) -> Result<&'a Track, Diagnostic> {
    sequence
        .tracks
        .iter()
        .find(|track| track.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "apply target track does not exist"))
}

fn item_track<'a>(sequence: &'a Sequence, id: &ItemId) -> Result<&'a Track, Diagnostic> {
    sequence
        .tracks
        .iter()
        .find(|track| track.clips.iter().any(|clip| clip.id == *id))
        .ok_or_else(|| operation_error(id.as_str(), "apply target item does not exist"))
}

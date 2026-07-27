use std::{cmp::Ordering, collections::BTreeSet};

use crate::*;

#[cfg(test)]
#[path = "insertion/tests.rs"]
mod tests;

use super::time_math::add;
use crate::edit::operations::{changed_tree, membership};
use crate::edit::{find_clip, find_clip_track_mut, operation_error, ChangeSet, MarkChanged};

pub(super) fn insert(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let EditOperation::InsertClip {
        sequence_id,
        track_id,
        clip,
        before_id,
        after_id,
    } = operation
    else {
        unreachable!("insert called with another operation")
    };
    reject_duplicate(project, &clip.id)?;
    if before_id.is_some() && after_id.is_some() {
        return Err(operation_error(
            track_id.as_str(),
            "insert accepts one neighbor",
        ));
    }
    let track = target_track(project, sequence_id, track_id)?;
    let index = relative_index(&track.clips, before_id.as_ref(), after_id.as_ref())?;
    track.clips.insert(index, (**clip).clone());
    changed_tree::clip(clip, changed);
    changed.track(track_id.clone());
    Ok(())
}

pub(super) fn ripple_insert(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let EditOperation::RippleInsert {
        sequence_id,
        track_id,
        clip,
    } = operation
    else {
        unreachable!("ripple_insert called with another operation")
    };
    reject_duplicate(project, &clip.id)?;
    let affected = ripple_members(project, sequence_id, track_id, clip.record_range.start)?;
    membership::ensure_unlocked(project, &affected)?;
    for id in affected {
        let track = find_clip_track_mut(project, &id)
            .ok_or_else(|| operation_error(id.as_str(), "ripple member does not exist"))?;
        let existing = track
            .clips
            .iter_mut()
            .find(|value| value.id == id)
            .ok_or_else(|| operation_error(id.as_str(), "ripple member does not exist"))?;
        existing.record_range.start = add(
            existing.record_range.start,
            clip.record_range.duration,
            id.as_str(),
        )?;
        changed.item(id);
        changed.track(track.id.clone());
        track.clips.sort_by(start_order);
    }
    let track = target_track(project, sequence_id, track_id)?;
    track.clips.push((**clip).clone());
    track.clips.sort_by(start_order);
    changed_tree::clip(clip, changed);
    changed.track(track_id.clone());
    Ok(())
}

fn ripple_members(
    project: &Project,
    sequence_id: &SequenceId,
    track_id: &TrackId,
    threshold: RationalTime,
) -> Result<Vec<ItemId>, Diagnostic> {
    let track = project
        .sequences
        .iter()
        .find(|value| value.id == *sequence_id)
        .and_then(|sequence| sequence.tracks.iter().find(|value| value.id == *track_id))
        .ok_or_else(|| operation_error(track_id.as_str(), "target track does not exist"))?;
    let seeds: Vec<_> = track
        .clips
        .iter()
        .filter(|clip| clip.record_range.start >= threshold)
        .map(|clip| clip.id.clone())
        .collect();
    let mut result = BTreeSet::new();
    for seed in seeds {
        let (_, connected) = membership::connected(project, &seed)?;
        result.extend(connected);
    }
    Ok(result.into_iter().collect())
}

fn reject_duplicate(project: &Project, id: &ItemId) -> Result<(), Diagnostic> {
    if find_clip(project, id).is_some() {
        Err(operation_error(
            id.as_str(),
            "inserted clip ID already exists",
        ))
    } else {
        Ok(())
    }
}

fn target_track<'a>(
    project: &'a mut Project,
    sequence_id: &SequenceId,
    track_id: &TrackId,
) -> Result<&'a mut Track, Diagnostic> {
    let track = project
        .sequences
        .iter_mut()
        .find(|sequence| sequence.id == *sequence_id)
        .and_then(|sequence| {
            sequence
                .tracks
                .iter_mut()
                .find(|track| track.id == *track_id)
        })
        .ok_or_else(|| operation_error(track_id.as_str(), "target track does not exist"))?;
    if track.state.locked {
        Err(operation_error(track_id.as_str(), "target track is locked"))
    } else {
        Ok(track)
    }
}

fn relative_index(
    clips: &[Clip],
    before_id: Option<&ItemId>,
    after_id: Option<&ItemId>,
) -> Result<usize, Diagnostic> {
    let Some(id) = before_id.or(after_id) else {
        return Ok(clips.len());
    };
    let index = clips
        .iter()
        .position(|clip| clip.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "neighbor clip does not exist"))?;
    Ok(index + usize::from(after_id.is_some()))
}

fn start_order(left: &Clip, right: &Clip) -> Ordering {
    left.record_range
        .start
        .partial_cmp(&right.record_range.start)
        .unwrap_or(Ordering::Equal)
}

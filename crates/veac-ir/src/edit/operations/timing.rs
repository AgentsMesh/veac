use std::{cmp::Ordering, collections::BTreeSet};

use crate::*;

mod split;
mod trim;

#[cfg(test)]
mod tests;

pub(super) use split::{split, split_one};
pub(super) use trim::trim;

use super::{
    membership,
    time_math::{add, subtract},
};
use crate::edit::operations::{apply_refs, changed_tree, source_time::shift_source};
use crate::edit::{find_clip, find_clip_track_mut, operation_error, ChangeSet, MarkChanged};

pub(super) fn remove(
    project: &mut Project,
    clip_id: &ItemId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let (sequence_id, members) = membership::connected(project, clip_id)?;
    membership::ensure_unlocked(project, &members)?;
    let removed: BTreeSet<_> = members.iter().cloned().collect();
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == sequence_id)
        .expect("membership returned an existing sequence");
    apply_refs::ensure_items_removable(sequence, &removed)?;
    for id in &members {
        let track = find_clip_track_mut(project, id)
            .ok_or_else(|| operation_error(id.as_str(), "clip does not exist"))?;
        let index = clip_index(track, id)?;
        let removed = track.clips.remove(index);
        changed_tree::clip(&removed, changed);
        changed.track(track.id.clone());
    }
    membership::detach_removed(project, &sequence_id, &removed, changed)?;
    Ok(())
}

pub(super) fn move_clip(
    project: &mut Project,
    clip_id: &ItemId,
    record_start: RationalTime,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let (_, members) = membership::connected(project, clip_id)?;
    membership::ensure_unlocked(project, &members)?;
    let current = find_clip(project, clip_id)
        .ok_or_else(|| operation_error(clip_id.as_str(), "clip does not exist"))?
        .record_range
        .start;
    let delta = subtract(record_start, current, clip_id.as_str())?;
    if delta.value == 0 {
        return Ok(());
    }
    for id in members {
        let track = find_clip_track_mut(project, &id)
            .ok_or_else(|| operation_error(id.as_str(), "clip does not exist"))?;
        let index = clip_index(track, &id)?;
        track.clips[index].record_range.start =
            add(track.clips[index].record_range.start, delta, id.as_str())?;
        changed.item(id);
        changed.track(track.id.clone());
        track.clips.sort_by(start_order);
    }
    Ok(())
}

pub(super) fn slip(
    project: &mut Project,
    clip_id: &ItemId,
    source_delta: RationalTime,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let members = membership::linked(project, clip_id)?;
    membership::ensure_unlocked(project, &members)?;
    for id in members {
        let track = find_clip_track_mut(project, &id)
            .ok_or_else(|| operation_error(id.as_str(), "clip does not exist"))?;
        let index = clip_index(track, &id)?;
        let mapping = track.clips[index]
            .source_mapping
            .as_mut()
            .ok_or_else(|| operation_error(id.as_str(), "slip requires linked media clips"))?;
        shift_source(mapping, source_delta, id.as_str())?;
        changed.item(id);
    }
    Ok(())
}

pub(super) fn unlocked_track<'a>(
    project: &'a mut Project,
    clip_id: &ItemId,
) -> Result<&'a mut Track, Diagnostic> {
    let track = find_clip_track_mut(project, clip_id)
        .ok_or_else(|| operation_error(clip_id.as_str(), "clip does not exist"))?;
    if track.state.locked {
        Err(operation_error(
            clip_id.as_str(),
            "clip belongs to a locked track",
        ))
    } else {
        Ok(track)
    }
}

pub(super) fn clip_index(track: &Track, id: &ItemId) -> Result<usize, Diagnostic> {
    track
        .clips
        .iter()
        .position(|clip| clip.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "clip does not exist"))
}

pub(super) fn require_positive(time: RationalTime, id: &ItemId) -> Result<(), Diagnostic> {
    if time.value > 0 {
        Ok(())
    } else {
        Err(operation_error(
            id.as_str(),
            "clip duration must stay positive",
        ))
    }
}

pub(super) fn negative(time: RationalTime, id: &ItemId) -> Result<RationalTime, Diagnostic> {
    super::time_math::subtract(
        RationalTime::zero(time.timescale)
            .map_err(|_| operation_error(id.as_str(), "invalid timebase"))?,
        time,
        id.as_str(),
    )
}

pub(super) fn start_order(left: &Clip, right: &Clip) -> Ordering {
    left.record_range
        .start
        .partial_cmp(&right.record_range.start)
        .unwrap_or(Ordering::Equal)
}

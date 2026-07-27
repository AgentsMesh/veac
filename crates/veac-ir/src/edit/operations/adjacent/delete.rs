use std::collections::BTreeSet;

use crate::*;

use super::{clip_index, negative, unlocked_track};
use crate::edit::operations::{apply_refs, changed_tree, membership, time_math::shift_from};
use crate::edit::{find_clip, operation_error, ChangeSet, MarkChanged};

#[cfg(test)]
mod tests;

pub(crate) fn ripple_delete(
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
    let mut ordered: Vec<_> = members
        .iter()
        .map(|id| {
            find_clip(project, id)
                .map(|clip| (id.clone(), clip.record_range.start))
                .ok_or_else(|| operation_error(id.as_str(), "clip does not exist"))
        })
        .collect::<Result<_, _>>()?;
    ordered.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| right.0.cmp(&left.0))
    });
    for (id, _) in ordered {
        delete_one(project, &id, changed)?;
    }
    membership::detach_removed(project, &sequence_id, &removed, changed)?;
    Ok(())
}

fn delete_one(
    project: &mut Project,
    clip_id: &ItemId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let track = unlocked_track(project, clip_id)?;
    let index = clip_index(track, clip_id)?;
    let removed = track.clips.remove(index);
    let end = removed
        .record_range
        .end()
        .map_err(|_| operation_error(clip_id.as_str(), "clip range is invalid"))?;
    let shift = negative(removed.record_range.duration, clip_id)?;
    shift_from(track, end, shift, &[], changed)?;
    changed_tree::clip(&removed, changed);
    changed.track(track.id.clone());
    Ok(())
}

use crate::*;

mod carve;
mod support;

#[cfg(test)]
mod tests;

use self::{
    carve::carve,
    support::{fragment_map, reject_related_overlaps, target_track, validate_fragment_sources},
};
use super::{apply_refs, relation_refs::rewrite_item_relations};
use crate::edit::{find_clip, operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let EditOperation::OverwriteClip {
        sequence_id,
        track_id,
        clip,
        split_fragments,
    } = operation
    else {
        unreachable!("overwrite dispatcher received another operation")
    };
    if find_clip(project, &clip.id).is_some() {
        return Err(operation_error(
            clip.id.as_str(),
            "overwrite clip ID already exists",
        ));
    }
    let fragments = fragment_map(project, split_fragments, &clip.id)?;
    let overwrite_end = clip
        .record_range
        .end()
        .map_err(|_| operation_error(clip.id.as_str(), "overwrite range is invalid"))?;
    reject_related_overlaps(
        project,
        sequence_id,
        track_id,
        clip.record_range.start,
        overwrite_end,
    )?;
    let sequence = project
        .sequences
        .iter()
        .find(|sequence| sequence.id == *sequence_id)
        .ok_or_else(|| operation_error(sequence_id.as_str(), "sequence does not exist"))?;
    let existing = sequence
        .tracks
        .iter()
        .find(|track| track.id == *track_id)
        .ok_or_else(|| operation_error(track_id.as_str(), "target track does not exist"))?;
    let removed = existing
        .clips
        .iter()
        .filter(|existing| {
            existing.record_range.start >= clip.record_range.start
                && existing
                    .record_range
                    .end()
                    .is_ok_and(|end| end <= overwrite_end)
        })
        .map(|existing| existing.id.clone())
        .collect();
    apply_refs::ensure_items_removable(sequence, &removed)?;
    let track = target_track(project, sequence_id, track_id)?;
    validate_fragment_sources(track, clip.record_range.start, overwrite_end, &fragments)?;
    let track_id = track.id.clone();
    let mut result = Vec::with_capacity(track.clips.len() + fragments.len() + 1);
    let mut rewrites = Vec::new();
    for existing in track.clips.drain(..) {
        carve(
            existing,
            clip.record_range.start,
            overwrite_end,
            &fragments,
            &mut result,
            &mut rewrites,
            changed,
        )?;
    }
    result.push((**clip).clone());
    result.sort_by(|left, right| {
        left.record_range
            .start
            .partial_cmp(&right.record_range.start)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| left.id.cmp(&right.id))
    });
    track.clips = result;
    changed.item(clip.id.clone());
    changed.track(track_id);
    rewrite_item_relations(project, sequence_id, &rewrites, changed)?;
    Ok(())
}

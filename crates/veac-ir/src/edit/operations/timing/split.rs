use crate::*;

use super::clip_index;
use crate::edit::operations::{
    curves::crop_clip,
    membership,
    relation_refs::{rewrite_item_relations, ItemTopology},
    time_math::*,
};
use crate::edit::{
    ensure_clip_unlocked, find_clip, find_clip_track_mut, operation_error, ChangeSet, MarkChanged,
};

#[cfg(test)]
mod tests;

pub(crate) fn split(
    project: &mut Project,
    clip_id: &ItemId,
    at: RationalTime,
    right_id: &ItemId,
    relation_fragments: &[RelationFragmentId],
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    membership::ensure_independent(project, clip_id)?;
    split_one(project, clip_id, at, right_id, relation_fragments, changed)
}

pub(crate) fn split_one(
    project: &mut Project,
    clip_id: &ItemId,
    at: RationalTime,
    right_id: &ItemId,
    relation_fragments: &[RelationFragmentId],
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    ensure_clip_unlocked(project, clip_id)?;
    let sequence_id = membership::sequence_id(project, clip_id)?;
    if find_clip(project, right_id).is_some() {
        return Err(operation_error(
            right_id.as_str(),
            "split result ID already exists",
        ));
    }
    let track = find_clip_track_mut(project, clip_id)
        .ok_or_else(|| operation_error(clip_id.as_str(), "clip does not exist"))?;
    let track_id = track.id.clone();
    let index = clip_index(track, clip_id)?;
    let original = track.clips[index].clone();
    let end = original
        .record_range
        .end()
        .map_err(|_| operation_error(clip_id.as_str(), "clip range is invalid"))?;
    if at <= original.record_range.start || at >= end {
        return Err(operation_error(
            clip_id.as_str(),
            "split point must be strictly inside the clip",
        ));
    }
    let left_duration = subtract(at, original.record_range.start, clip_id.as_str())?;
    let right_duration = subtract(end, at, clip_id.as_str())?;
    let mut left = original.clone();
    let mut right = original;
    right.id = right_id.clone();
    crop_clip(
        &mut left,
        RationalTime::zero(at.timescale)
            .map_err(|_| operation_error(clip_id.as_str(), "invalid timebase"))?,
        left_duration,
        clip_id,
        false,
        changed,
    )?;
    crop_clip(
        &mut right,
        left_duration,
        right_duration,
        right_id,
        true,
        changed,
    )?;
    left.record_range.duration = left_duration;
    right.record_range = TimeRange {
        start: at,
        duration: right_duration,
    };
    split_source(
        &mut left.source_mapping,
        &mut right.source_mapping,
        left_duration,
        right_duration,
        clip_id.as_str(),
    )?;
    track.clips[index] = left;
    track.clips.insert(index + 1, right);
    changed.item(clip_id.clone());
    changed.item(right_id.clone());
    changed.track(track_id);
    rewrite_item_relations(
        project,
        &sequence_id,
        &[ItemTopology::Split {
            item_id: clip_id.clone(),
            right_item_id: right_id.clone(),
            relation_fragments: relation_fragments.to_vec(),
        }],
        changed,
    )?;
    Ok(())
}

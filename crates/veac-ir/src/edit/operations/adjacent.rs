use crate::*;

mod delete;

#[cfg(test)]
mod defense_tests;
#[cfg(test)]
mod tests;

pub(super) use delete::ripple_delete;

use super::{
    curves::crop_clip,
    membership,
    time_math::{add, subtract, trim_source},
    timing::{clip_index, require_positive},
};
use crate::edit::{find_clip_track_mut, operation_error, ChangeSet, MarkChanged};

pub(super) fn roll(
    project: &mut Project,
    left_id: &ItemId,
    right_id: &ItemId,
    delta: RationalTime,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    membership::ensure_independent(project, left_id)?;
    membership::ensure_independent(project, right_id)?;
    let track = unlocked_track(project, left_id)?;
    let left_index = clip_index(track, left_id)?;
    let right_index = clip_index(track, right_id)?;
    if right_index != left_index + 1
        || track.clips[left_index].record_range.end().ok()
            != Some(track.clips[right_index].record_range.start)
    {
        return Err(operation_error(
            left_id.as_str(),
            "roll requires adjacent contiguous clips in one track",
        ));
    }
    let left_duration = add(
        track.clips[left_index].record_range.duration,
        delta,
        left_id.as_str(),
    )?;
    let right_duration = subtract(
        track.clips[right_index].record_range.duration,
        delta,
        right_id.as_str(),
    )?;
    require_positive(left_duration, left_id)?;
    require_positive(right_duration, right_id)?;
    let zero = RationalTime::zero(delta.timescale)
        .map_err(|_| operation_error(left_id.as_str(), "invalid timebase"))?;
    crop_clip(
        &mut track.clips[left_index],
        zero,
        left_duration,
        left_id,
        false,
        changed,
    )?;
    crop_clip(
        &mut track.clips[right_index],
        delta,
        right_duration,
        right_id,
        false,
        changed,
    )?;
    track.clips[left_index].record_range.duration = left_duration;
    track.clips[right_index].record_range.start = add(
        track.clips[right_index].record_range.start,
        delta,
        right_id.as_str(),
    )?;
    track.clips[right_index].record_range.duration = right_duration;
    if let Some(mapping) = &mut track.clips[left_index].source_mapping {
        trim_source(mapping, TrimEdge::Out, delta, left_id.as_str())?;
    }
    if let Some(mapping) = &mut track.clips[right_index].source_mapping {
        trim_source(mapping, TrimEdge::In, delta, right_id.as_str())?;
    }
    changed.item(left_id.clone());
    changed.item(right_id.clone());
    changed.track(track.id.clone());
    Ok(())
}

pub(super) fn slide(
    project: &mut Project,
    clip_id: &ItemId,
    delta: RationalTime,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    membership::ensure_independent(project, clip_id)?;
    let track = unlocked_track(project, clip_id)?;
    let index = clip_index(track, clip_id)?;
    if index == 0 || index + 1 >= track.clips.len() {
        return Err(operation_error(
            clip_id.as_str(),
            "slide requires clips on both sides",
        ));
    }
    let previous = index - 1;
    let next = index + 1;
    let contiguous = track.clips[previous].record_range.end().ok()
        == Some(track.clips[index].record_range.start)
        && track.clips[index].record_range.end().ok() == Some(track.clips[next].record_range.start);
    if !contiguous {
        return Err(operation_error(
            clip_id.as_str(),
            "slide requires contiguous neighboring clips",
        ));
    }
    let previous_duration = add(
        track.clips[previous].record_range.duration,
        delta,
        track.clips[previous].id.as_str(),
    )?;
    let next_duration = subtract(
        track.clips[next].record_range.duration,
        delta,
        track.clips[next].id.as_str(),
    )?;
    require_positive(previous_duration, &track.clips[previous].id)?;
    require_positive(next_duration, &track.clips[next].id)?;
    let previous_id = track.clips[previous].id.clone();
    let next_id = track.clips[next].id.clone();
    let zero = RationalTime::zero(delta.timescale)
        .map_err(|_| operation_error(clip_id.as_str(), "invalid timebase"))?;
    crop_clip(
        &mut track.clips[previous],
        zero,
        previous_duration,
        &previous_id,
        false,
        changed,
    )?;
    crop_clip(
        &mut track.clips[next],
        delta,
        next_duration,
        &next_id,
        false,
        changed,
    )?;
    track.clips[previous].record_range.duration = previous_duration;
    track.clips[index].record_range.start = add(
        track.clips[index].record_range.start,
        delta,
        clip_id.as_str(),
    )?;
    track.clips[next].record_range.start = add(
        track.clips[next].record_range.start,
        delta,
        next_id.as_str(),
    )?;
    track.clips[next].record_range.duration = next_duration;
    if let Some(mapping) = &mut track.clips[previous].source_mapping {
        trim_source(mapping, TrimEdge::Out, delta, previous_id.as_str())?;
    }
    if let Some(mapping) = &mut track.clips[next].source_mapping {
        trim_source(mapping, TrimEdge::In, delta, next_id.as_str())?;
    }
    changed.item(track.clips[previous].id.clone());
    changed.item(clip_id.clone());
    changed.item(next_id);
    changed.track(track.id.clone());
    Ok(())
}

fn unlocked_track<'a>(
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

fn negative(time: RationalTime, id: &ItemId) -> Result<RationalTime, Diagnostic> {
    let zero = RationalTime::zero(time.timescale)
        .map_err(|_| operation_error(id.as_str(), "invalid timebase"))?;
    subtract(zero, time, id.as_str())
}

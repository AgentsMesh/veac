use std::collections::BTreeSet;

use crate::*;

use super::{clip_index, negative, require_positive, start_order, unlocked_track};
use crate::edit::operations::{curves::crop_clip, membership, time_math::*};
use crate::edit::{operation_error, ChangeSet, MarkChanged};

#[cfg(test)]
mod tests;

pub(crate) fn trim(
    project: &mut Project,
    clip_id: &ItemId,
    edge: TrimEdge,
    delta: RationalTime,
    ripple: bool,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let (_, members) = membership::connected(project, clip_id)?;
    membership::ensure_unlocked(project, &members)?;
    if ripple {
        let tracks: BTreeSet<_> = members
            .iter()
            .map(|id| membership::track_id(project, id))
            .collect::<Result<_, _>>()?;
        if tracks.len() != members.len() {
            return Err(operation_error(
                clip_id.as_str(),
                "ripple trim cannot target multiple grouped clips on one track",
            ));
        }
    }
    for id in members {
        trim_one(project, &id, edge, delta, ripple, changed)?;
    }
    Ok(())
}

fn trim_one(
    project: &mut Project,
    clip_id: &ItemId,
    edge: TrimEdge,
    delta: RationalTime,
    ripple: bool,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let track = unlocked_track(project, clip_id)?;
    let track_id = track.id.clone();
    let index = clip_index(track, clip_id)?;
    let original = track.clips[index].record_range;
    let new_duration = match edge {
        TrimEdge::In => subtract(original.duration, delta, clip_id.as_str())?,
        TrimEdge::Out => add(original.duration, delta, clip_id.as_str())?,
    };
    require_positive(new_duration, clip_id)?;
    let curve_start = match edge {
        TrimEdge::In => delta,
        TrimEdge::Out => RationalTime::zero(delta.timescale)
            .map_err(|_| crate::edit::operation_error(clip_id.as_str(), "invalid timebase"))?,
    };
    crop_clip(
        &mut track.clips[index],
        curve_start,
        new_duration,
        clip_id,
        false,
        changed,
    )?;
    if let Some(mapping) = &mut track.clips[index].source_mapping {
        trim_source(mapping, edge, delta, clip_id.as_str())?;
    }
    if edge == TrimEdge::In && !ripple {
        track.clips[index].record_range.start = add(original.start, delta, clip_id.as_str())?;
    }
    track.clips[index].record_range.duration = new_duration;
    changed.item(clip_id.clone());
    changed.track(track_id);
    if ripple {
        let shift = match edge {
            TrimEdge::In => negative(delta, clip_id)?,
            TrimEdge::Out => delta,
        };
        shift_from(
            track,
            original.end().map_err(|_| {
                crate::edit::operation_error(clip_id.as_str(), "clip range is invalid")
            })?,
            shift,
            &[clip_id],
            changed,
        )?;
    }
    track.clips.sort_by(start_order);
    Ok(())
}

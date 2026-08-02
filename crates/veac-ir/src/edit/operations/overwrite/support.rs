use std::collections::{BTreeMap, BTreeSet};

use crate::*;

use crate::edit::operations::membership;
use crate::edit::{find_clip, operation_error};

#[cfg(test)]
mod tests;

pub(super) fn fragment_map(
    project: &Project,
    values: &[OverwriteFragment],
    inserted_id: &ItemId,
) -> Result<BTreeMap<ItemId, OverwriteFragment>, Diagnostic> {
    let mut result = BTreeMap::new();
    let mut right_ids = BTreeSet::new();
    for value in values {
        let duplicate = find_clip(project, &value.right_fragment_id).is_some()
            || value.right_fragment_id == *inserted_id
            || !right_ids.insert(value.right_fragment_id.clone())
            || result
                .insert(value.source_clip_id.clone(), value.clone())
                .is_some();
        if duplicate {
            return Err(operation_error(
                value.right_fragment_id.as_str(),
                "overwrite fragment IDs must be new and unambiguous",
            ));
        }
    }
    Ok(result)
}

pub(super) fn validate_fragment_sources(
    track: &Track,
    start: RationalTime,
    end: RationalTime,
    fragments: &BTreeMap<ItemId, OverwriteFragment>,
) -> Result<(), Diagnostic> {
    let needed: BTreeSet<_> = track
        .clips
        .iter()
        .filter(|clip| {
            clip.record_range.start < start
                && clip.record_range.end().is_ok_and(|clip_end| clip_end > end)
        })
        .map(|clip| clip.id.clone())
        .collect();
    let supplied: BTreeSet<_> = fragments.keys().cloned().collect();
    if needed == supplied {
        Ok(())
    } else {
        Err(operation_error(
            track.id.as_str(),
            "overwrite split fragments must exactly match clips spanning the overwrite range",
        ))
    }
}

pub(super) fn target_track<'a>(
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

pub(super) fn reject_related_overlaps(
    project: &Project,
    sequence_id: &SequenceId,
    track_id: &TrackId,
    start: RationalTime,
    end: RationalTime,
) -> Result<(), Diagnostic> {
    let sequence = project
        .sequences
        .iter()
        .find(|value| value.id == *sequence_id)
        .ok_or_else(|| operation_error(sequence_id.as_str(), "sequence does not exist"))?;
    let track = sequence
        .tracks
        .iter()
        .find(|value| value.id == *track_id)
        .ok_or_else(|| operation_error(track_id.as_str(), "target track does not exist"))?;
    for clip in &track.clips {
        let overlaps = clip.record_range.start < end
            && clip
                .record_range
                .end()
                .is_ok_and(|clip_end| clip_end > start);
        let (groups, links) = membership::relationship_ids(project, &sequence.id, &clip.id);
        if overlaps && (!groups.is_empty() || !links.is_empty()) {
            return Err(operation_error(
                clip.id.as_str(),
                "overwrite cannot implicitly sever a clip group or AV link",
            ));
        }
    }
    Ok(())
}

use std::collections::BTreeMap;

use crate::edit::{operation_error, ChangeSet, MarkChanged};
use crate::*;

use super::super::{
    changed_tree,
    curves::crop_clip,
    relation_refs::ItemTopology,
    time_math::{subtract, trim_source},
};

#[cfg(test)]
#[path = "carve/tests.rs"]
mod tests;

pub(super) fn carve(
    mut clip: Clip,
    start: RationalTime,
    end: RationalTime,
    fragments: &BTreeMap<ItemId, OverwriteFragment>,
    result: &mut Vec<Clip>,
    rewrites: &mut Vec<ItemTopology>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let old_end = clip
        .record_range
        .end()
        .map_err(|_| operation_error(clip.id.as_str(), "clip range is invalid"))?;
    if old_end <= start || clip.record_range.start >= end {
        result.push(clip);
    } else if clip.record_range.start >= start && old_end <= end {
        rewrites.push(ItemTopology::Removed {
            item_id: clip.id.clone(),
        });
        changed_tree::clip(&clip, changed);
    } else if clip.record_range.start < start && old_end > end {
        split(&clip, start, end, fragments, result, rewrites, changed)?;
    } else if clip.record_range.start < start {
        keep_left(&mut clip, start, rewrites, changed)?;
        result.push(clip);
    } else {
        keep_right(&mut clip, end, old_end, rewrites, changed)?;
        result.push(clip);
    }
    Ok(())
}

fn keep_left(
    clip: &mut Clip,
    start: RationalTime,
    rewrites: &mut Vec<ItemTopology>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let id = clip.id.clone();
    let old_duration = clip.record_range.duration;
    let duration = subtract(start, clip.record_range.start, id.as_str())?;
    crop_clip(clip, zero(start, &id)?, duration, &id, false, changed)?;
    clip.record_range.duration = duration;
    trim_mapping(
        clip,
        TrimEdge::Out,
        subtract(duration, old_duration, id.as_str())?,
    )?;
    changed.item(id.clone());
    rewrites.push(ItemTopology::KeptLeft { item_id: id });
    Ok(())
}

fn keep_right(
    clip: &mut Clip,
    end: RationalTime,
    old_end: RationalTime,
    rewrites: &mut Vec<ItemTopology>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let id = clip.id.clone();
    let offset = subtract(end, clip.record_range.start, id.as_str())?;
    let duration = subtract(old_end, end, id.as_str())?;
    crop_clip(clip, offset, duration, &id, false, changed)?;
    trim_mapping(clip, TrimEdge::In, offset)?;
    clip.record_range = TimeRange {
        start: end,
        duration,
    };
    changed.item(id.clone());
    rewrites.push(ItemTopology::KeptRight {
        item_id: id,
        offset,
    });
    Ok(())
}

fn split(
    clip: &Clip,
    start: RationalTime,
    end: RationalTime,
    fragments: &BTreeMap<ItemId, OverwriteFragment>,
    result: &mut Vec<Clip>,
    rewrites: &mut Vec<ItemTopology>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let source_id = clip.id.clone();
    let fragment = &fragments[&source_id];
    split_around(
        clip.clone(),
        start,
        end,
        &fragment.right_fragment_id,
        result,
        changed,
    )?;
    rewrites.push(ItemTopology::Split {
        item_id: source_id,
        right_item_id: fragment.right_fragment_id.clone(),
        relation_fragments: fragment.relation_fragments.clone(),
    });
    Ok(())
}

fn split_around(
    mut left: Clip,
    start: RationalTime,
    end: RationalTime,
    right_id: &ItemId,
    result: &mut Vec<Clip>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let source_id = left.id.clone();
    let old_duration = left.record_range.duration;
    let old_end = left
        .record_range
        .end()
        .map_err(|_| operation_error(source_id.as_str(), "clip range is invalid"))?;
    let left_duration = subtract(start, left.record_range.start, source_id.as_str())?;
    let offset = subtract(end, left.record_range.start, source_id.as_str())?;
    let right_duration = subtract(old_end, end, right_id.as_str())?;
    let mut right = left.clone();
    right.id = right_id.clone();
    crop_clip(
        &mut left,
        zero(start, &source_id)?,
        left_duration,
        &source_id,
        false,
        changed,
    )?;
    crop_clip(&mut right, offset, right_duration, right_id, true, changed)?;
    left.record_range.duration = left_duration;
    trim_mapping(
        &mut left,
        TrimEdge::Out,
        subtract(left_duration, old_duration, source_id.as_str())?,
    )?;
    trim_mapping(&mut right, TrimEdge::In, offset)?;
    right.record_range = TimeRange {
        start: end,
        duration: right_duration,
    };
    changed.item(source_id);
    changed.item(right_id.clone());
    result.extend([left, right]);
    Ok(())
}

fn trim_mapping(clip: &mut Clip, edge: TrimEdge, delta: RationalTime) -> Result<(), Diagnostic> {
    if let Some(mapping) = &mut clip.source_mapping {
        trim_source(mapping, edge, delta, clip.id.as_str())?;
    }
    Ok(())
}

fn zero(time: RationalTime, id: &ItemId) -> Result<RationalTime, Diagnostic> {
    RationalTime::zero(time.timescale)
        .map_err(|_| operation_error(id.as_str(), "overwrite timebase is invalid"))
}

use crate::*;

use super::evaluate::{at, CurveValue};
use crate::edit::{operation_error, ChangeSet, MarkChanged};

mod interpolation;

#[cfg(test)]
mod tests;

#[cfg(test)]
#[path = "slice/value_type_tests.rs"]
mod value_type_tests;

pub(super) fn crop_animatable<T: CurveValue>(
    value: &mut Animatable<T>,
    start: RationalTime,
    end: RationalTime,
    owner: &ItemId,
    path: &str,
    reidentify: bool,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let Animatable::Keyframes { keyframes } = value else {
        return Ok(());
    };
    let old = keyframes.clone();
    let start_value = at(&old, start)
        .ok_or_else(|| operation_error(owner.as_str(), "cannot slice an empty animation curve"))?;
    let end_value = at(&old, end)
        .ok_or_else(|| operation_error(owner.as_str(), "cannot slice an empty animation curve"))?;
    let duration = super::super::time_math::subtract(end, start, owner.as_str())?;
    let mut sliced = Vec::new();
    sliced.push(boundary(
        &old,
        start,
        RationalTime::zero(start.timescale)
            .map_err(|_| operation_error(owner.as_str(), "animation timebase is invalid"))?,
        start_value,
        owner,
        path,
        "start",
        reidentify,
    ));
    for keyframe in old.iter().filter(|key| key.time > start && key.time < end) {
        let mut keyframe = keyframe.clone();
        keyframe.time = super::super::time_math::subtract(keyframe.time, start, owner.as_str())?;
        if reidentify {
            keyframe.id =
                super::super::clone_ids::derived_keyframe(owner, &keyframe.id, path, "inside");
        }
        sliced.push(keyframe);
    }
    sliced.push(boundary(
        &old, end, duration, end_value, owner, path, "end", reidentify,
    ));
    sliced.dedup_by(|right, left| right.time == left.time);
    reslice_interpolations(&old, &mut sliced, start, owner)?;
    mark_ids(&old, &sliced, changed);
    *keyframes = sliced;
    Ok(())
}

fn reslice_interpolations<T>(
    old: &[Keyframe<T>],
    sliced: &mut [Keyframe<T>],
    source_start: RationalTime,
    owner: &ItemId,
) -> Result<(), Diagnostic> {
    for index in 0..sliced.len().saturating_sub(1) {
        let start = super::super::time_math::add(source_start, sliced[index].time, owner.as_str())?;
        let end =
            super::super::time_math::add(source_start, sliced[index + 1].time, owner.as_str())?;
        sliced[index].interpolation = interpolation::between(old, start, end).map_err(|_| {
            operation_error(
                owner.as_str(),
                "animation easing cannot be preserved across the requested slice",
            )
        })?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn boundary<T: Clone>(
    old: &[Keyframe<T>],
    source_time: RationalTime,
    new_time: RationalTime,
    value: T,
    owner: &ItemId,
    path: &str,
    side: &str,
    reidentify: bool,
) -> Keyframe<T> {
    let exact = old.iter().find(|key| key.time == source_time);
    let interpolation = exact
        .map(|key| key.interpolation.clone())
        .or_else(|| {
            old.iter()
                .rev()
                .find(|key| key.time < source_time)
                .map(|key| key.interpolation.clone())
        })
        .unwrap_or(Interpolation::Hold);
    let base_id = exact.map(|key| &key.id);
    let id = if !reidentify {
        base_id.cloned().unwrap_or_else(|| {
            super::super::clone_ids::synthetic_keyframe(owner, path, side, source_time)
        })
    } else {
        let fallback = super::super::clone_ids::synthetic_keyframe(owner, path, side, source_time);
        super::super::clone_ids::derived_keyframe(owner, base_id.unwrap_or(&fallback), path, side)
    };
    Keyframe {
        id,
        time: new_time,
        value,
        interpolation,
    }
}

fn mark_ids<T>(old: &[Keyframe<T>], new: &[Keyframe<T>], changed: &mut ChangeSet) {
    let unchanged = old.len() == new.len()
        && old
            .iter()
            .zip(new)
            .all(|(left, right)| left.id == right.id && left.time == right.time);
    if !unchanged {
        for id in old.iter().chain(new).map(|key| key.id.clone()) {
            changed.keyframe(id);
        }
    }
}

use crate::*;

use super::{mutate, search::curve_has};

#[cfg(test)]
mod tests;

pub(super) fn has(clip: &Clip, id: &KeyframeId) -> bool {
    animation(clip).is_some_and(|value| {
        curve_has(&value.reveal, id)
            || value
                .highlight
                .as_ref()
                .is_some_and(|highlight| curve_has(&highlight.progress, id))
            || curve_has(&value.opacity, id)
            || curve_has(&value.transform.position_offset, id)
            || curve_has(&value.transform.scale, id)
            || curve_has(&value.transform.rotation_degrees, id)
    })
}

pub(super) fn remove(clip: &mut Clip, id: &KeyframeId) -> Result<bool, Diagnostic> {
    let Some(value) = animation_mut(clip) else {
        return Ok(false);
    };
    if mutate::remove(&mut value.reveal, id)? {
        return Ok(true);
    }
    if let Some(highlight) = &mut value.highlight {
        if mutate::remove(&mut highlight.progress, id)? {
            return Ok(true);
        }
    }
    Ok(mutate::remove(&mut value.opacity, id)?
        || mutate::remove(&mut value.transform.position_offset, id)?
        || mutate::remove(&mut value.transform.scale, id)?
        || mutate::remove(&mut value.transform.rotation_degrees, id)?)
}

pub(super) fn move_keyframe(
    clip: &mut Clip,
    id: &KeyframeId,
    time: RationalTime,
) -> Result<Option<bool>, Diagnostic> {
    let Some(value) = animation_mut(clip) else {
        return Ok(None);
    };
    if let Some(result) = mutate::move_time(&mut value.reveal, id, time)? {
        return Ok(Some(result));
    }
    if let Some(highlight) = &mut value.highlight {
        if let Some(result) = mutate::move_time(&mut highlight.progress, id, time)? {
            return Ok(Some(result));
        }
    }
    if let Some(result) = mutate::move_time(&mut value.opacity, id, time)? {
        return Ok(Some(result));
    }
    if let Some(result) = mutate::move_time(&mut value.transform.position_offset, id, time)? {
        return Ok(Some(result));
    }
    if let Some(result) = mutate::move_time(&mut value.transform.scale, id, time)? {
        return Ok(Some(result));
    }
    mutate::move_time(&mut value.transform.rotation_degrees, id, time)
}

fn animation(clip: &Clip) -> Option<&TextAnimation> {
    match &clip.source {
        ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } => {
            style.animation.as_ref()
        }
        _ => None,
    }
}

fn animation_mut(clip: &mut Clip) -> Option<&mut TextAnimation> {
    match &mut clip.source {
        ClipSource::Text { style, .. } | ClipSource::Caption { style, .. } => {
            style.animation.as_mut()
        }
        _ => None,
    }
}

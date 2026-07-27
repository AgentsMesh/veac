use crate::*;

use super::mutate;

pub(super) fn has(visual: &VisualProperties, id: &KeyframeId) -> bool {
    visual.masks.iter().any(|mask| {
        super::search::curve_has(&mask.position, id)
            || super::search::curve_has(&mask.scale, id)
            || super::search::curve_has(&mask.rotation_degrees, id)
            || super::search::curve_has(&mask.feather_pixels, id)
            || super::search::curve_has(&mask.expansion_pixels, id)
    })
}

pub(super) fn remove(visual: &mut VisualProperties, id: &KeyframeId) -> Result<bool, Diagnostic> {
    for mask in &mut visual.masks {
        if mutate::remove(&mut mask.position, id)?
            || mutate::remove(&mut mask.scale, id)?
            || mutate::remove(&mut mask.rotation_degrees, id)?
            || mutate::remove(&mut mask.feather_pixels, id)?
            || mutate::remove(&mut mask.expansion_pixels, id)?
        {
            return Ok(true);
        }
    }
    Ok(false)
}

pub(super) fn move_time(
    visual: &mut VisualProperties,
    id: &KeyframeId,
    time: RationalTime,
) -> Result<Option<bool>, Diagnostic> {
    for mask in &mut visual.masks {
        if let Some(value) = mutate::move_time(&mut mask.position, id, time)? {
            return Ok(Some(value));
        }
        if let Some(value) = mutate::move_time(&mut mask.scale, id, time)? {
            return Ok(Some(value));
        }
        if let Some(value) = mutate::move_time(&mut mask.rotation_degrees, id, time)? {
            return Ok(Some(value));
        }
        if let Some(value) = mutate::move_time(&mut mask.feather_pixels, id, time)? {
            return Ok(Some(value));
        }
        if let Some(value) = mutate::move_time(&mut mask.expansion_pixels, id, time)? {
            return Ok(Some(value));
        }
    }
    Ok(None)
}

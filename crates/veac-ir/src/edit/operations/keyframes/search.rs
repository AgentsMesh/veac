use crate::*;

use super::mutate;
use crate::edit::operation_error;

pub(super) struct MoveResult {
    pub changed: bool,
    pub effect_id: Option<EffectId>,
}

pub(super) fn project_has(project: &Project, id: &KeyframeId) -> bool {
    project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .any(|clip| clip_has(clip, id))
}

pub(super) fn curve_has<T>(curve: &Animatable<T>, id: &KeyframeId) -> bool {
    curve
        .keyframes()
        .is_some_and(|keys| keys.iter().any(|key| key.id == *id))
}

pub(super) fn remove(clip: &mut Clip, id: &KeyframeId) -> Result<Option<EffectId>, Diagnostic> {
    if remove_visual(clip, id)? || remove_audio(clip, id)? || super::text::remove(clip, id)? {
        return Ok(None);
    }
    for effect in &mut clip.effects {
        for parameter in effect.parameters.values_mut() {
            if let ParameterValue::NumberCurve { value } = parameter {
                if mutate::remove(value, id)? {
                    return Ok(Some(effect.id.clone()));
                }
            }
        }
    }
    Err(operation_error(
        id.as_str(),
        "keyframe does not exist on the clip",
    ))
}

pub(super) fn move_keyframe(
    clip: &mut Clip,
    id: &KeyframeId,
    time: RationalTime,
) -> Result<MoveResult, Diagnostic> {
    if let Some(changed) = move_visual(clip, id, time)? {
        return Ok(MoveResult {
            changed,
            effect_id: None,
        });
    }
    if let Some(changed) = move_audio(clip, id, time)? {
        return Ok(MoveResult {
            changed,
            effect_id: None,
        });
    }
    if let Some(changed) = super::text::move_keyframe(clip, id, time)? {
        return Ok(MoveResult {
            changed,
            effect_id: None,
        });
    }
    for effect in &mut clip.effects {
        for parameter in effect.parameters.values_mut() {
            if let ParameterValue::NumberCurve { value } = parameter {
                if let Some(changed) = mutate::move_time(value, id, time)? {
                    return Ok(MoveResult {
                        changed,
                        effect_id: Some(effect.id.clone()),
                    });
                }
            }
        }
    }
    Err(operation_error(
        id.as_str(),
        "keyframe does not exist on the clip",
    ))
}

fn clip_has(clip: &Clip, id: &KeyframeId) -> bool {
    let visual = clip.visual.as_ref().is_some_and(|value| {
        curve_has(&value.transform.position, id)
            || curve_has(&value.transform.scale, id)
            || curve_has(&value.transform.rotation_degrees, id)
            || value
                .transform
                .crop
                .as_ref()
                .is_some_and(|crop| curve_has(crop, id))
            || curve_has(&value.opacity, id)
            || super::masks::has(value, id)
    });
    let audio = clip
        .audio
        .as_ref()
        .is_some_and(|value| curve_has(&value.gain, id) || curve_has(&value.pan, id));
    let text = super::text::has(clip, id);
    visual
        || audio
        || text
        || clip.effects.iter().any(|effect| {
            effect.parameters.values().any(|parameter| {
                matches!(parameter, ParameterValue::NumberCurve { value } if curve_has(value, id))
            })
        })
}

fn remove_visual(clip: &mut Clip, id: &KeyframeId) -> Result<bool, Diagnostic> {
    let Some(value) = &mut clip.visual else {
        return Ok(false);
    };
    if super::masks::remove(value, id)? {
        return Ok(true);
    }
    if let Some(crop) = &mut value.transform.crop {
        if mutate::remove(crop, id)? {
            return Ok(true);
        }
    }
    Ok(mutate::remove(&mut value.transform.position, id)?
        || mutate::remove(&mut value.transform.scale, id)?
        || mutate::remove(&mut value.transform.rotation_degrees, id)?
        || mutate::remove(&mut value.opacity, id)?)
}

fn remove_audio(clip: &mut Clip, id: &KeyframeId) -> Result<bool, Diagnostic> {
    let Some(value) = &mut clip.audio else {
        return Ok(false);
    };
    Ok(mutate::remove(&mut value.gain, id)? || mutate::remove(&mut value.pan, id)?)
}

fn move_visual(
    clip: &mut Clip,
    id: &KeyframeId,
    time: RationalTime,
) -> Result<Option<bool>, Diagnostic> {
    let Some(value) = &mut clip.visual else {
        return Ok(None);
    };
    if let Some(result) = super::masks::move_time(value, id, time)? {
        return Ok(Some(result));
    }
    if let Some(result) = mutate::move_time(&mut value.transform.position, id, time)? {
        return Ok(Some(result));
    }
    if let Some(result) = mutate::move_time(&mut value.transform.scale, id, time)? {
        return Ok(Some(result));
    }
    if let Some(result) = mutate::move_time(&mut value.transform.rotation_degrees, id, time)? {
        return Ok(Some(result));
    }
    if let Some(crop) = &mut value.transform.crop {
        if let Some(result) = mutate::move_time(crop, id, time)? {
            return Ok(Some(result));
        }
    }
    mutate::move_time(&mut value.opacity, id, time)
}

fn move_audio(
    clip: &mut Clip,
    id: &KeyframeId,
    time: RationalTime,
) -> Result<Option<bool>, Diagnostic> {
    let Some(value) = &mut clip.audio else {
        return Ok(None);
    };
    if let Some(result) = mutate::move_time(&mut value.gain, id, time)? {
        return Ok(Some(result));
    }
    mutate::move_time(&mut value.pan, id, time)
}

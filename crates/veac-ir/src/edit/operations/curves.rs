mod evaluate;
mod masks;
mod slice;
mod text;

use crate::*;

use self::slice::crop_animatable;
use crate::edit::{operation_error, ChangeSet, MarkChanged};

#[cfg(test)]
mod defense_tests;

pub(super) fn crop_clip(
    clip: &mut Clip,
    start: RationalTime,
    duration: RationalTime,
    owner: &ItemId,
    reidentify: bool,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let end = start
        .checked_add(duration)
        .map_err(|_| operation_error(owner.as_str(), "curve slice range is invalid"))?;
    if let Some(visual) = &mut clip.visual {
        crop_animatable(
            &mut visual.transform.position,
            start,
            end,
            owner,
            "visual_position",
            reidentify,
            changed,
        )?;
        crop_animatable(
            &mut visual.transform.scale,
            start,
            end,
            owner,
            "visual_scale",
            reidentify,
            changed,
        )?;
        crop_animatable(
            &mut visual.transform.rotation_degrees,
            start,
            end,
            owner,
            "visual_rotation",
            reidentify,
            changed,
        )?;
        if let Some(crop) = &mut visual.transform.crop {
            crop_animatable(crop, start, end, owner, "visual_crop", reidentify, changed)?;
        }
        crop_animatable(
            &mut visual.opacity,
            start,
            end,
            owner,
            "visual_opacity",
            reidentify,
            changed,
        )?;
        masks::crop(visual, start, end, owner, reidentify, changed)?;
    }
    if let Some(audio) = &mut clip.audio {
        crop_animatable(
            &mut audio.gain,
            start,
            end,
            owner,
            "audio_gain",
            reidentify,
            changed,
        )?;
        crop_animatable(
            &mut audio.pan,
            start,
            end,
            owner,
            "audio_pan",
            reidentify,
            changed,
        )?;
    }
    text::crop(clip, start, end, owner, reidentify, changed)?;
    crop_effects(clip, start, end, owner, reidentify, changed)
}

fn crop_effects(
    clip: &mut Clip,
    start: RationalTime,
    end: RationalTime,
    owner: &ItemId,
    reidentify: bool,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let mut retained = Vec::with_capacity(clip.effects.len());
    for mut effect in clip.effects.drain(..) {
        let before = effect.clone();
        let old_id = effect.id.clone();
        if !crop_enable_range(&mut effect, start, end, owner)? {
            changed.effect(old_id);
            continue;
        }
        for parameter in EffectParameter::ALL {
            if let Some(value) = effect.effect.curve_mut(parameter) {
                crop_animatable(
                    value,
                    start,
                    end,
                    owner,
                    &format!("effect_{}_{}", old_id, parameter.name()),
                    reidentify,
                    changed,
                )?;
            }
        }
        if reidentify {
            effect.id = super::clone_ids::derived_effect(owner, &old_id);
            changed.effect(effect.id.clone());
        }
        if effect != before {
            changed.effect(old_id);
        }
        retained.push(effect);
    }
    clip.effects = retained;
    Ok(())
}

fn crop_enable_range(
    effect: &mut EffectInstance,
    start: RationalTime,
    end: RationalTime,
    owner: &ItemId,
) -> Result<bool, Diagnostic> {
    let Some(range) = effect.enable_range else {
        return Ok(true);
    };
    let range_end = range
        .end()
        .map_err(|_| operation_error(owner.as_str(), "effect range is invalid"))?;
    let overlap_start = if range.start > start {
        range.start
    } else {
        start
    };
    let overlap_end = if range_end < end { range_end } else { end };
    if overlap_end <= overlap_start {
        return Ok(false);
    }
    effect.enable_range = Some(TimeRange {
        start: super::time_math::subtract(overlap_start, start, owner.as_str())?,
        duration: super::time_math::subtract(overlap_end, overlap_start, owner.as_str())?,
    });
    Ok(true)
}

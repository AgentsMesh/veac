use crate::*;

use super::changed_tree;
use crate::edit::{ensure_clip_unlocked, find_clip_mut, operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match operation {
        EditOperation::SetVisualProperty { clip_id, property } => {
            visual(project, clip_id, property, changed)
        }
        EditOperation::SetAudioProperty { clip_id, property } => {
            audio(project, clip_id, property, changed)
        }
        EditOperation::EditEffectParameter { edit } => effect(project, edit, changed),
        _ => unreachable!("property dispatcher received another operation"),
    }
}

fn visual(
    project: &mut Project,
    clip_id: &ItemId,
    property: &VisualProperty,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let clip = mutable_clip(project, clip_id)?;
    let visual = clip
        .visual
        .as_mut()
        .ok_or_else(|| operation_error(clip_id.as_str(), "clip has no visual properties"))?;
    let did_change = match property {
        VisualProperty::Placement(value) => replace(&mut visual.placement, *value),
        VisualProperty::Frame(value) => replace(&mut visual.frame, *value),
        VisualProperty::Position(value) => curve(&mut visual.transform.position, value, changed),
        VisualProperty::Scale(value) => curve(&mut visual.transform.scale, value, changed),
        VisualProperty::Shear(value) => replace(&mut visual.transform.shear, *value),
        VisualProperty::FlipHorizontal(value) => {
            replace(&mut visual.transform.flip_horizontal, *value)
        }
        VisualProperty::FlipVertical(value) => replace(&mut visual.transform.flip_vertical, *value),
        VisualProperty::RotationDegrees(value) => {
            curve(&mut visual.transform.rotation_degrees, value, changed)
        }
        VisualProperty::Anchor(value) => replace(&mut visual.transform.anchor, *value),
        VisualProperty::Crop(value) => replace(&mut visual.transform.crop, value.clone()),
        VisualProperty::Opacity(value) => curve(&mut visual.opacity, value, changed),
        VisualProperty::Compositing(value) => replace(&mut visual.compositing, value.clone()),
        VisualProperty::Masks(value) => masks(&mut visual.masks, value, changed),
        VisualProperty::Card(value) => replace(&mut visual.card, value.clone()),
        VisualProperty::ColorPipeline(value) => replace(&mut visual.color_pipeline, value.clone()),
    };
    if did_change {
        changed.item(clip_id.clone());
    }
    Ok(())
}

fn audio(
    project: &mut Project,
    clip_id: &ItemId,
    property: &AudioProperty,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let clip = mutable_clip(project, clip_id)?;
    let audio = clip
        .audio
        .as_mut()
        .ok_or_else(|| operation_error(clip_id.as_str(), "clip has no audio properties"))?;
    let did_change = match property {
        AudioProperty::Gain(value) => curve(&mut audio.gain, value, changed),
        AudioProperty::Pan(value) => curve(&mut audio.pan, value, changed),
        AudioProperty::Muted(value) => replace(&mut audio.muted, *value),
        AudioProperty::Normalize(value) => replace(&mut audio.normalize, *value),
        AudioProperty::PitchPolicy(value) => replace(&mut audio.pitch_policy, *value),
        AudioProperty::Processors(value) => replace(&mut audio.processors, value.clone()),
        AudioProperty::Crossfade(value) => replace(&mut audio.crossfade, *value),
    };
    if did_change {
        changed.item(clip_id.clone());
    }
    Ok(())
}

fn effect(
    project: &mut Project,
    edit: &EffectParameterEdit,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let (clip_id, effect_id, name) = match edit {
        EffectParameterEdit::Set {
            clip_id,
            effect_id,
            name,
            ..
        }
        | EffectParameterEdit::Remove {
            clip_id,
            effect_id,
            name,
        } => (clip_id, effect_id, name),
    };
    let clip = mutable_clip(project, clip_id)?;
    let effect = clip
        .effects
        .iter_mut()
        .find(|value| value.id == *effect_id)
        .ok_or_else(|| operation_error(effect_id.as_str(), "effect does not exist on the clip"))?;
    let did_change = match edit {
        EffectParameterEdit::Set { value, .. } => match effect.parameters.get(name) {
            Some(current) if current == value => false,
            _ => {
                if let Some(old) = effect.parameters.insert(name.clone(), value.clone()) {
                    mark_parameter(&old, changed);
                }
                mark_parameter(value, changed);
                true
            }
        },
        EffectParameterEdit::Remove { .. } => {
            let old = effect.parameters.remove(name).ok_or_else(|| {
                operation_error(effect_id.as_str(), "effect parameter does not exist")
            })?;
            mark_parameter(&old, changed);
            true
        }
    };
    if did_change {
        changed.item(clip_id.clone());
        changed.effect(effect_id.clone());
    }
    Ok(())
}

fn mutable_clip<'a>(project: &'a mut Project, id: &ItemId) -> Result<&'a mut Clip, Diagnostic> {
    ensure_clip_unlocked(project, id)?;
    find_clip_mut(project, id).ok_or_else(|| operation_error(id.as_str(), "clip does not exist"))
}

fn curve<T: Clone + PartialEq>(
    target: &mut Animatable<T>,
    value: &Animatable<T>,
    changed: &mut ChangeSet,
) -> bool {
    if target == value {
        return false;
    }
    changed_tree::curve(target, changed);
    changed_tree::curve(value, changed);
    *target = value.clone();
    true
}

fn replace<T: PartialEq>(target: &mut T, value: T) -> bool {
    if *target == value {
        false
    } else {
        *target = value;
        true
    }
}

fn masks(target: &mut Vec<Mask>, value: &[Mask], changed: &mut ChangeSet) -> bool {
    if target == value {
        return false;
    }
    for mask in target.iter().chain(value) {
        changed_tree::mask(mask, changed);
    }
    *target = value.to_vec();
    true
}

fn mark_parameter(value: &ParameterValue, changed: &mut ChangeSet) {
    if let ParameterValue::NumberCurve { value } = value {
        changed_tree::curve(value, changed);
    }
}

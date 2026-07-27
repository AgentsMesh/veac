use crate::*;

use crate::edit::operations::changed_tree;
use crate::edit::{ensure_clip_unlocked, find_clip_mut, operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match operation {
        EditOperation::AddEffect {
            clip_id,
            effect,
            before_id,
            after_id,
        } => add(project, clip_id, effect, before_id, after_id, changed),
        EditOperation::RemoveEffect { clip_id, effect_id } => {
            remove(project, clip_id, effect_id, changed)
        }
        EditOperation::MoveEffect {
            clip_id,
            effect_id,
            before_id,
            after_id,
        } => move_effect(project, clip_id, effect_id, before_id, after_id, changed),
        _ => unreachable!("effect dispatcher received another variant"),
    }
}

fn add(
    project: &mut Project,
    clip_id: &ItemId,
    effect: &EffectInstance,
    before_id: &Option<EffectId>,
    after_id: &Option<EffectId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    ensure_one_neighbor(before_id, after_id, clip_id)?;
    if project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .flat_map(|track| &track.clips)
        .flat_map(|clip| &clip.effects)
        .any(|existing| existing.id == effect.id)
    {
        return Err(operation_error(
            effect.id.as_str(),
            "effect ID already exists",
        ));
    }
    let clip = mutable_clip(project, clip_id)?;
    let index = relative_index(&clip.effects, before_id.as_ref(), after_id.as_ref())?;
    clip.effects.insert(index, effect.clone());
    changed.item(clip_id.clone());
    changed_tree::effect(effect, changed);
    Ok(())
}

fn remove(
    project: &mut Project,
    clip_id: &ItemId,
    effect_id: &EffectId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let clip = mutable_clip(project, clip_id)?;
    let index = effect_index(&clip.effects, effect_id)?;
    let removed = clip.effects.remove(index);
    changed.item(clip_id.clone());
    changed_tree::effect(&removed, changed);
    Ok(())
}

fn move_effect(
    project: &mut Project,
    clip_id: &ItemId,
    effect_id: &EffectId,
    before_id: &Option<EffectId>,
    after_id: &Option<EffectId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    ensure_one_neighbor(before_id, after_id, clip_id)?;
    if before_id.as_ref() == Some(effect_id) || after_id.as_ref() == Some(effect_id) {
        return Err(operation_error(
            effect_id.as_str(),
            "effect cannot be positioned relative to itself",
        ));
    }
    let clip = mutable_clip(project, clip_id)?;
    let old_order: Vec<_> = clip
        .effects
        .iter()
        .map(|effect| effect.id.clone())
        .collect();
    let index = effect_index(&clip.effects, effect_id)?;
    let effect = clip.effects.remove(index);
    let target = relative_index(&clip.effects, before_id.as_ref(), after_id.as_ref())?;
    clip.effects.insert(target, effect);
    let new_order: Vec<_> = clip
        .effects
        .iter()
        .map(|effect| effect.id.clone())
        .collect();
    if old_order != new_order {
        changed.item(clip_id.clone());
        changed.effect(effect_id.clone());
    }
    Ok(())
}

fn mutable_clip<'a>(
    project: &'a mut Project,
    clip_id: &ItemId,
) -> Result<&'a mut Clip, Diagnostic> {
    ensure_clip_unlocked(project, clip_id)?;
    find_clip_mut(project, clip_id)
        .ok_or_else(|| operation_error(clip_id.as_str(), "clip does not exist"))
}

fn ensure_one_neighbor(
    before_id: &Option<EffectId>,
    after_id: &Option<EffectId>,
    clip_id: &ItemId,
) -> Result<(), Diagnostic> {
    if before_id.is_some() && after_id.is_some() {
        Err(operation_error(
            clip_id.as_str(),
            "effect positioning accepts one neighbor",
        ))
    } else {
        Ok(())
    }
}

fn effect_index(effects: &[EffectInstance], id: &EffectId) -> Result<usize, Diagnostic> {
    effects
        .iter()
        .position(|effect| effect.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "effect does not exist on the clip"))
}

fn relative_index(
    effects: &[EffectInstance],
    before_id: Option<&EffectId>,
    after_id: Option<&EffectId>,
) -> Result<usize, Diagnostic> {
    let Some(id) = before_id.or(after_id) else {
        return Ok(effects.len());
    };
    Ok(effect_index(effects, id)? + usize::from(after_id.is_some()))
}

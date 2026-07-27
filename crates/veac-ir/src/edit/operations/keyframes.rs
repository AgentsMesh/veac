mod masks;
mod mutate;
mod search;
mod targets;
mod text;

use crate::*;

use crate::edit::{ensure_clip_unlocked, find_clip_mut, operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    edit: &KeyframeEdit,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match edit {
        KeyframeEdit::UpsertNumber {
            clip_id,
            target,
            keyframe,
        } => upsert_number(project, clip_id, target, keyframe, changed),
        KeyframeEdit::UpsertPoint {
            clip_id,
            target,
            keyframe,
        } => upsert_point(project, clip_id, *target, keyframe, changed),
        KeyframeEdit::UpsertVec2 {
            clip_id,
            target,
            keyframe,
        } => upsert_vec2(project, clip_id, target, keyframe, changed),
        KeyframeEdit::UpsertRect {
            clip_id,
            target,
            keyframe,
        } => upsert_rect(project, clip_id, *target, keyframe, changed),
        KeyframeEdit::Remove {
            clip_id,
            keyframe_id,
        } => remove(project, clip_id, keyframe_id, changed),
        KeyframeEdit::Move {
            clip_id,
            keyframe_id,
            time,
        } => move_keyframe(project, clip_id, keyframe_id, *time, changed),
    }
}

fn upsert_number(
    project: &mut Project,
    clip_id: &ItemId,
    target: &NumberCurveTarget,
    keyframe: &Keyframe<f64>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let global = search::project_has(project, &keyframe.id);
    let clip = mutable_clip(project, clip_id)?;
    let (curve, effect_id) = targets::number(clip, target)?;
    reject_foreign_id(global, search::curve_has(curve, &keyframe.id), &keyframe.id)?;
    if mutate::upsert(curve, keyframe.clone())? {
        mark(clip_id, &keyframe.id, effect_id, changed);
    }
    Ok(())
}

fn upsert_point(
    project: &mut Project,
    clip_id: &ItemId,
    target: PointCurveTarget,
    keyframe: &Keyframe<Point>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let global = search::project_has(project, &keyframe.id);
    let clip = mutable_clip(project, clip_id)?;
    let curve = targets::point(clip, target)?;
    reject_foreign_id(global, search::curve_has(curve, &keyframe.id), &keyframe.id)?;
    if mutate::upsert(curve, keyframe.clone())? {
        mark(clip_id, &keyframe.id, None, changed);
    }
    Ok(())
}

fn upsert_vec2(
    project: &mut Project,
    clip_id: &ItemId,
    target: &Vec2CurveTarget,
    keyframe: &Keyframe<Vec2>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let global = search::project_has(project, &keyframe.id);
    let clip = mutable_clip(project, clip_id)?;
    let curve = targets::vec2(clip, target)?;
    reject_foreign_id(global, search::curve_has(curve, &keyframe.id), &keyframe.id)?;
    if mutate::upsert(curve, keyframe.clone())? {
        mark(clip_id, &keyframe.id, None, changed);
    }
    Ok(())
}

fn upsert_rect(
    project: &mut Project,
    clip_id: &ItemId,
    target: RectCurveTarget,
    keyframe: &Keyframe<Rect>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let global = search::project_has(project, &keyframe.id);
    let clip = mutable_clip(project, clip_id)?;
    let curve = targets::rect(clip, target)?;
    reject_foreign_id(global, search::curve_has(curve, &keyframe.id), &keyframe.id)?;
    if mutate::upsert(curve, keyframe.clone())? {
        mark(clip_id, &keyframe.id, None, changed);
    }
    Ok(())
}

fn remove(
    project: &mut Project,
    clip_id: &ItemId,
    id: &KeyframeId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let clip = mutable_clip(project, clip_id)?;
    let effect = search::remove(clip, id)?;
    mark(clip_id, id, effect, changed);
    Ok(())
}

fn move_keyframe(
    project: &mut Project,
    clip_id: &ItemId,
    id: &KeyframeId,
    time: RationalTime,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let clip = mutable_clip(project, clip_id)?;
    let result = search::move_keyframe(clip, id, time)?;
    if result.changed {
        mark(clip_id, id, result.effect_id, changed);
    }
    Ok(())
}

fn mutable_clip<'a>(project: &'a mut Project, id: &ItemId) -> Result<&'a mut Clip, Diagnostic> {
    ensure_clip_unlocked(project, id)?;
    find_clip_mut(project, id).ok_or_else(|| operation_error(id.as_str(), "clip does not exist"))
}

fn reject_foreign_id(global: bool, local: bool, id: &KeyframeId) -> Result<(), Diagnostic> {
    if global && !local {
        Err(operation_error(
            id.as_str(),
            "keyframe ID already exists on another curve",
        ))
    } else {
        Ok(())
    }
}

fn mark(
    clip_id: &ItemId,
    keyframe_id: &KeyframeId,
    effect_id: Option<EffectId>,
    changed: &mut ChangeSet,
) {
    changed.item(clip_id.clone());
    changed.keyframe(keyframe_id.clone());
    if let Some(id) = effect_id {
        changed.effect(id);
    }
}

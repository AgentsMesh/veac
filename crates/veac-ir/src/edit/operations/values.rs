use crate::*;

use crate::edit::operations::changed_tree;
use crate::edit::operations::membership;
use crate::edit::{ensure_clip_unlocked, find_clip_mut, operation_error, ChangeSet, MarkChanged};

#[path = "values/transition.rs"]
mod transition_edit;

pub(super) fn apply(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if let EditOperation::SetClipEnabled { clip_id, enabled } = operation {
        return set_enabled(project, clip_id, *enabled, changed);
    }
    if let EditOperation::SetTransition {
        clip_id,
        transition,
    } = operation
    {
        return transition_edit::set(project, clip_id, transition, changed);
    }
    let clip_id = operation_clip_id(operation);
    ensure_clip_unlocked(project, clip_id)?;
    let clip = find_clip_mut(project, clip_id)
        .ok_or_else(|| operation_error(clip_id.as_str(), "clip does not exist"))?;
    let did_change = match operation {
        EditOperation::ReplaceSource {
            source,
            source_mapping,
            ..
        } if clip.source != **source || clip.source_mapping != *source_mapping => {
            clip.source = source.as_ref().clone();
            clip.source_mapping = source_mapping.clone();
            true
        }
        EditOperation::SetSourceMapping { source_mapping, .. }
            if clip.source_mapping.as_ref() != Some(source_mapping) =>
        {
            if !matches!(clip.source, ClipSource::Media { .. }) {
                return Err(operation_error(
                    clip_id.as_str(),
                    "source mapping target is not media",
                ));
            }
            clip.source_mapping = Some(source_mapping.clone());
            true
        }
        EditOperation::SetText { text, .. } => set_text(clip, text, clip_id)?,
        EditOperation::SetTemplateState {
            replaceable,
            template_editable_text,
            ..
        } if clip.replaceable != *replaceable
            || clip.template_editable_text != *template_editable_text =>
        {
            clip.replaceable = replaceable.clone();
            clip.template_editable_text = *template_editable_text;
            true
        }
        EditOperation::SetClipEnabled { enabled, .. } if clip.enabled != *enabled => {
            clip.enabled = *enabled;
            true
        }
        EditOperation::SetVisual { visual, .. } if clip.visual != *visual => {
            mark_visuals(clip.visual.as_ref(), visual.as_ref(), changed);
            clip.visual = visual.clone();
            true
        }
        EditOperation::SetAudio { audio, .. } if clip.audio != *audio => {
            mark_audio(clip.audio.as_ref(), audio.as_ref(), changed);
            clip.audio = audio.clone();
            true
        }
        _ => false,
    };
    if did_change {
        changed.item(clip_id.clone());
    }
    Ok(())
}

fn set_enabled(
    project: &mut Project,
    clip_id: &ItemId,
    enabled: bool,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let (_, members) = membership::connected(project, clip_id)?;
    membership::ensure_unlocked(project, &members)?;
    for id in members {
        let clip = find_clip_mut(project, &id)
            .ok_or_else(|| operation_error(id.as_str(), "clip does not exist"))?;
        if clip.enabled != enabled {
            clip.enabled = enabled;
            changed.item(id);
        }
    }
    Ok(())
}

fn mark_visuals(
    old: Option<&VisualProperties>,
    new: Option<&VisualProperties>,
    changed: &mut ChangeSet,
) {
    for value in old.into_iter().chain(new) {
        changed_tree::visual(value, changed);
    }
}

fn mark_audio(
    old: Option<&AudioProperties>,
    new: Option<&AudioProperties>,
    changed: &mut ChangeSet,
) {
    for value in old.into_iter().chain(new) {
        changed_tree::audio(value, changed);
    }
}

fn operation_clip_id(operation: &EditOperation) -> &ItemId {
    match operation {
        EditOperation::ReplaceSource { clip_id, .. }
        | EditOperation::SetSourceMapping { clip_id, .. }
        | EditOperation::SetText { clip_id, .. }
        | EditOperation::SetTemplateState { clip_id, .. }
        | EditOperation::SetClipEnabled { clip_id, .. }
        | EditOperation::SetVisual { clip_id, .. }
        | EditOperation::SetAudio { clip_id, .. }
        | EditOperation::SetTransition { clip_id, .. } => clip_id,
        _ => unreachable!("value operation dispatcher received another variant"),
    }
}

fn set_text(clip: &mut Clip, text: &str, clip_id: &ItemId) -> Result<bool, Diagnostic> {
    let current = match &mut clip.source {
        ClipSource::Text { text, .. } | ClipSource::Caption { text, .. } => text,
        _ => {
            return Err(operation_error(
                clip_id.as_str(),
                "set_text target is not text",
            ))
        }
    };
    if current == text {
        Ok(false)
    } else {
        current.clear();
        current.push_str(text);
        Ok(true)
    }
}

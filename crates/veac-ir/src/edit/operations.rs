mod adjacent;
mod annotations;
mod apply_refs;
mod changed_tree;
mod clone_ids;
mod curves;
mod effects;
mod insertion;
mod keyframes;
mod linked_split;
mod membership;
mod multicam;
mod overwrite;
mod properties;
mod relation_refs;
mod source_curve;
mod source_time;
mod structure;
mod text_properties;
mod time_math;
mod timing;
mod values;

use super::ChangeSet;
use crate::*;

pub(super) fn apply_operation(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match operation {
        EditOperation::InsertClip { .. } => insertion::insert(project, operation, changed),
        EditOperation::RippleInsert { .. } => insertion::ripple_insert(project, operation, changed),
        EditOperation::OverwriteClip { .. } => overwrite::apply(project, operation, changed),
        EditOperation::RemoveClip { clip_id } => timing::remove(project, clip_id, changed),
        EditOperation::RippleDelete { clip_id } => {
            adjacent::ripple_delete(project, clip_id, changed)
        }
        EditOperation::MoveClip {
            clip_id,
            record_start,
        } => timing::move_clip(project, clip_id, *record_start, changed),
        EditOperation::TrimClip {
            clip_id,
            edge,
            delta,
            ripple,
        } => timing::trim(project, clip_id, *edge, *delta, *ripple, changed),
        EditOperation::SplitClip {
            clip_id,
            at,
            right_clip_id,
            relation_fragments,
        } => timing::split(
            project,
            clip_id,
            *at,
            right_clip_id,
            relation_fragments,
            changed,
        ),
        EditOperation::SplitLinked { .. } => linked_split::apply(project, operation, changed),
        EditOperation::SlipClip {
            clip_id,
            source_delta,
        } => timing::slip(project, clip_id, *source_delta, changed),
        EditOperation::RollEdit {
            left_clip_id,
            right_clip_id,
            delta,
        } => adjacent::roll(project, left_clip_id, right_clip_id, *delta, changed),
        EditOperation::SlideClip { clip_id, delta } => {
            adjacent::slide(project, clip_id, *delta, changed)
        }
        EditOperation::ReplaceSource { .. }
        | EditOperation::SetSourceMapping { .. }
        | EditOperation::SetText { .. }
        | EditOperation::SetTemplateState { .. }
        | EditOperation::SetClipEnabled { .. }
        | EditOperation::SetVisual { .. }
        | EditOperation::SetAudio { .. }
        | EditOperation::SetTransition { .. } => values::apply(project, operation, changed),
        EditOperation::SetMulticamSwitches { .. } | EditOperation::SetMulticamGroup { .. } => {
            multicam::apply(project, operation, changed)
        }
        EditOperation::InsertAnnotation { .. }
        | EditOperation::SetAnnotation { .. }
        | EditOperation::RemoveAnnotation { .. } => annotations::apply(project, operation, changed),
        EditOperation::AddEffect { .. }
        | EditOperation::RemoveEffect { .. }
        | EditOperation::MoveEffect { .. } => effects::apply(project, operation, changed),
        EditOperation::SetVisualProperty { .. }
        | EditOperation::SetAudioProperty { .. }
        | EditOperation::EditEffectParameter { .. } => {
            properties::apply(project, operation, changed)
        }
        EditOperation::SetTextProperty { clip_id, property } => {
            text_properties::apply(project, clip_id, property, changed)
        }
        EditOperation::EditKeyframe { edit } => keyframes::apply(project, edit, changed),
        EditOperation::EditStructure { edit } => structure::apply(project, edit, changed),
    }
}

mod anchors;
mod apply;
mod insert;
mod references;
mod relation;
mod remove;
mod set;

use crate::*;

use crate::edit::ChangeSet;

pub(super) fn apply(
    project: &mut Project,
    edit: &StructureEdit,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match edit {
        StructureEdit::InsertApply { .. }
        | StructureEdit::SetApply { .. }
        | StructureEdit::RemoveApply { .. }
        | StructureEdit::MoveApply { .. } => apply::apply(project, edit, changed),
        StructureEdit::InsertRelation { .. }
        | StructureEdit::SetRelation { .. }
        | StructureEdit::RemoveRelation { .. } => relation::apply(project, edit, changed),
        StructureEdit::InsertMaterial { .. }
        | StructureEdit::InsertSequence { .. }
        | StructureEdit::InsertTrack { .. }
        | StructureEdit::InsertMulticamGroup { .. }
        | StructureEdit::InsertOutput { .. } => insert::apply(project, edit, changed),
        StructureEdit::RemoveMaterial { .. }
        | StructureEdit::RemoveSequence { .. }
        | StructureEdit::RemoveTrack { .. }
        | StructureEdit::RemoveMulticamGroup { .. }
        | StructureEdit::RemoveOutput { .. } => remove::apply(project, edit, changed),
        _ => set::apply(project, edit, changed),
    }
}

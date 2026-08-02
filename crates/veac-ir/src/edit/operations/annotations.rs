use crate::*;

use crate::edit::{operation_error, ChangeSet, MarkChanged};

pub(super) fn apply(
    project: &mut Project,
    operation: &EditOperation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match operation {
        EditOperation::InsertAnnotation { annotation } => insert(project, annotation, changed),
        EditOperation::SetAnnotation { annotation } => set(project, annotation, changed),
        EditOperation::RemoveAnnotation { annotation_id } => {
            remove(project, annotation_id, changed)
        }
        _ => unreachable!("annotation dispatcher received another operation"),
    }
}

fn insert(
    project: &mut Project,
    annotation: &Annotation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match project
        .annotations
        .binary_search_by(|value| value.id.cmp(&annotation.id))
    {
        Ok(_) => Err(operation_error(
            annotation.id.as_str(),
            "annotation ID already exists",
        )),
        Err(index) => {
            project.annotations.insert(index, annotation.clone());
            changed.annotation(annotation.id.clone());
            Ok(())
        }
    }
}

fn set(
    project: &mut Project,
    annotation: &Annotation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let value = project
        .annotations
        .iter_mut()
        .find(|value| value.id == annotation.id)
        .ok_or_else(|| operation_error(annotation.id.as_str(), "annotation does not exist"))?;
    if value != annotation {
        *value = annotation.clone();
        changed.annotation(annotation.id.clone());
    }
    Ok(())
}

fn remove(
    project: &mut Project,
    id: &AnnotationId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let index = project
        .annotations
        .iter()
        .position(|value| value.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "annotation does not exist"))?;
    project.annotations.remove(index);
    changed.annotation(id.clone());
    Ok(())
}

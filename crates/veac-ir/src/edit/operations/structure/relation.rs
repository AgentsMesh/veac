use crate::*;

#[cfg(test)]
#[path = "relation/tests.rs"]
mod tests;

use super::super::relation_refs::ensure_relation_unlocked;
use crate::edit::{operation_error, ChangeSet, MarkChanged, StructureEdit};

pub(super) fn apply(
    project: &mut Project,
    edit: &StructureEdit,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match edit {
        StructureEdit::InsertRelation { relation } => insert(project, relation, changed),
        StructureEdit::SetRelation { relation } => set(project, relation, changed),
        StructureEdit::RemoveRelation { relation_id } => remove(project, relation_id, changed),
        _ => unreachable!(),
    }
}

fn insert(
    project: &mut Project,
    relation: &Relation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if project.relations.iter().any(|item| item.id == relation.id) {
        return Err(operation_error(
            relation.id.as_str(),
            "relation ID already exists",
        ));
    }
    ensure_relation_unlocked(project, relation)?;
    let mut scoped = scoped_relations(project, &relation.sequence_id);
    scoped.push(relation.clone());
    validate_scoped(project, &relation.sequence_id, &scoped)?;
    project.relations.push(relation.clone());
    mark(project, &relation.id, changed);
    Ok(())
}

fn set(
    project: &mut Project,
    relation: &Relation,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let Some(index) = project
        .relations
        .iter()
        .position(|item| item.id == relation.id)
    else {
        return Err(operation_error(
            relation.id.as_str(),
            "relation does not exist",
        ));
    };
    let old = project.relations[index].clone();
    if old.sequence_id != relation.sequence_id {
        return Err(operation_error(
            relation.id.as_str(),
            "relation sequence ID is immutable",
        ));
    }
    ensure_relation_unlocked(project, &old)?;
    ensure_relation_unlocked(project, relation)?;
    let mut scoped = scoped_relations(project, &relation.sequence_id);
    let position = scoped
        .iter()
        .position(|item| item.id == relation.id)
        .unwrap();
    scoped[position] = relation.clone();
    validate_scoped(project, &relation.sequence_id, &scoped)?;
    if old != *relation {
        project.relations[index] = relation.clone();
        mark(project, &relation.id, changed);
    }
    Ok(())
}

fn remove(
    project: &mut Project,
    relation_id: &RelationId,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    let Some(index) = project
        .relations
        .iter()
        .position(|item| item.id == *relation_id)
    else {
        return Err(operation_error(
            relation_id.as_str(),
            "relation does not exist",
        ));
    };
    let relation = project.relations[index].clone();
    ensure_relation_unlocked(project, &relation)?;
    project.relations.remove(index);
    mark(project, relation_id, changed);
    Ok(())
}

fn validate_scoped(
    project: &Project,
    sequence_id: &SequenceId,
    relations: &[Relation],
) -> Result<(), Diagnostic> {
    let sequence = sequence(project, sequence_id)?;
    validate_sequence_relations(sequence, relations).map_err(|errors| {
        operation_error(
            sequence_id.as_str(),
            &format!("relation graph is invalid: {errors}"),
        )
    })
}

fn scoped_relations(project: &Project, sequence_id: &SequenceId) -> Vec<Relation> {
    project
        .relations
        .iter()
        .filter(|relation| relation.sequence_id == *sequence_id)
        .cloned()
        .collect()
}

fn sequence<'a>(project: &'a Project, id: &SequenceId) -> Result<&'a Sequence, Diagnostic> {
    project
        .sequences
        .iter()
        .find(|item| item.id == *id)
        .ok_or_else(|| operation_error(id.as_str(), "relation sequence does not exist"))
}

fn mark(project: &Project, relation_id: &RelationId, changed: &mut ChangeSet) {
    changed.relation(relation_id.clone());
    changed.project(project.id.clone());
}

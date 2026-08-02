use std::collections::BTreeSet;

use crate::edit::operations::{apply_refs, changed_tree, relation_refs};
use crate::edit::{find_apply, operation_error, ChangeSet, MarkChanged};
use crate::*;

#[cfg(test)]
#[path = "apply/tests.rs"]
mod tests;

use super::anchors;

pub(super) fn apply(
    project: &mut Project,
    edit: &StructureEdit,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    match edit {
        StructureEdit::InsertApply {
            sequence_id,
            apply,
            before_id,
            after_id,
        } => insert(project, sequence_id, apply, before_id, after_id, changed),
        StructureEdit::SetApply { apply_id, apply } => set(project, apply_id, apply, changed),
        StructureEdit::RemoveApply { apply_id } => remove(project, apply_id, changed),
        StructureEdit::MoveApply {
            apply_id,
            before_id,
            after_id,
        } => move_apply(project, apply_id, before_id, after_id, changed),
        _ => unreachable!("apply dispatcher received another structure edit"),
    }
}

fn insert(
    project: &mut Project,
    sequence_id: &SequenceId,
    value: &Apply,
    before: &Option<ApplyId>,
    after: &Option<ApplyId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if find_apply(project, &value.id).is_some() {
        return Err(operation_error(
            value.id.as_str(),
            "apply ID already exists",
        ));
    }
    let sequence_index = project
        .sequences
        .iter()
        .position(|sequence| sequence.id == *sequence_id)
        .ok_or_else(|| operation_error(sequence_id.as_str(), "sequence does not exist"))?;
    let sequence = &project.sequences[sequence_index];
    apply_refs::ensure_target_unlocked(sequence, &value.target, &BTreeSet::new())?;
    let index = anchors::index(
        &sequence.applies,
        before.as_ref(),
        after.as_ref(),
        |apply| &apply.id,
        value.id.as_str(),
    )?;
    project.sequences[sequence_index]
        .applies
        .insert(index, value.clone());
    changed_tree::apply(value, changed);
    changed.sequence(sequence_id.clone());
    Ok(())
}

fn set(
    project: &mut Project,
    id: &ApplyId,
    value: &Apply,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if value.id != *id {
        return Err(operation_error(
            id.as_str(),
            "replacement apply ID must match",
        ));
    }
    let (sequence_index, apply_index) = locate(project, id)?;
    let sequence = &project.sequences[sequence_index];
    let old = &sequence.applies[apply_index];
    if old == value {
        return Ok(());
    }
    apply_refs::ensure_target_unlocked(sequence, &old.target, &BTreeSet::new())?;
    apply_refs::ensure_target_unlocked(sequence, &value.target, &BTreeSet::new())?;
    let old = old.clone();
    project.sequences[sequence_index].applies[apply_index] = value.clone();
    changed_tree::apply(&old, changed);
    changed_tree::apply(value, changed);
    changed.sequence(project.sequences[sequence_index].id.clone());
    Ok(())
}

fn remove(project: &mut Project, id: &ApplyId, changed: &mut ChangeSet) -> Result<(), Diagnostic> {
    let (sequence_index, apply_index) = locate(project, id)?;
    let sequence_id = project.sequences[sequence_index].id.clone();
    let value = project.sequences[sequence_index].applies[apply_index].clone();
    apply_refs::ensure_target_unlocked(
        &project.sequences[sequence_index],
        &value.target,
        &BTreeSet::new(),
    )?;
    relation_refs::remove_apply(project, &sequence_id, id, changed)?;
    project.sequences[sequence_index]
        .applies
        .remove(apply_index);
    changed_tree::apply(&value, changed);
    changed.sequence(sequence_id);
    Ok(())
}

fn move_apply(
    project: &mut Project,
    id: &ApplyId,
    before: &Option<ApplyId>,
    after: &Option<ApplyId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if before.as_ref() == Some(id) || after.as_ref() == Some(id) {
        return Err(operation_error(
            id.as_str(),
            "apply cannot anchor to itself",
        ));
    }
    let (sequence_index, apply_index) = locate(project, id)?;
    let sequence = &project.sequences[sequence_index];
    apply_refs::ensure_target_unlocked(
        sequence,
        &sequence.applies[apply_index].target,
        &BTreeSet::new(),
    )?;
    let before_order: Vec<_> = sequence
        .applies
        .iter()
        .map(|apply| apply.id.clone())
        .collect();
    let value = project.sequences[sequence_index]
        .applies
        .remove(apply_index);
    let index = anchors::index(
        &project.sequences[sequence_index].applies,
        before.as_ref(),
        after.as_ref(),
        |apply| &apply.id,
        id.as_str(),
    )?;
    project.sequences[sequence_index]
        .applies
        .insert(index, value);
    let sequence = &project.sequences[sequence_index];
    let after_order: Vec<_> = sequence
        .applies
        .iter()
        .map(|apply| apply.id.clone())
        .collect();
    if before_order != after_order {
        changed.apply(id.clone());
        changed.sequence(sequence.id.clone());
    }
    Ok(())
}

fn locate(project: &Project, id: &ApplyId) -> Result<(usize, usize), Diagnostic> {
    project
        .sequences
        .iter()
        .enumerate()
        .find_map(|(sequence_index, sequence)| {
            sequence
                .applies
                .iter()
                .position(|apply| apply.id == *id)
                .map(|apply_index| (sequence_index, apply_index))
        })
        .ok_or_else(|| operation_error(id.as_str(), "apply does not exist"))
}

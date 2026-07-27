use crate::*;

#[cfg(test)]
#[path = "sequence/tests.rs"]
mod tests;

use super::super::anchors;
use crate::edit::operations::changed_tree;
use crate::edit::{operation_error, ChangeSet, MarkChanged};

pub(super) fn insert_sequence(
    project: &mut Project,
    sequence: &Sequence,
    relations: &[Relation],
    before: &Option<SequenceId>,
    after: &Option<SequenceId>,
    changed: &mut ChangeSet,
) -> Result<(), Diagnostic> {
    if project.sequences.iter().any(|item| item.id == sequence.id) {
        return Err(operation_error(
            sequence.id.as_str(),
            "sequence ID already exists",
        ));
    }
    let index = anchors::index(
        &project.sequences,
        before.as_ref(),
        after.as_ref(),
        |item| &item.id,
        sequence.id.as_str(),
    )?;
    for relation in relations {
        if project.relations.iter().any(|item| item.id == relation.id) {
            return Err(operation_error(
                relation.id.as_str(),
                "relation ID already exists",
            ));
        }
    }
    validate_sequence_relations(sequence, relations).map_err(|errors| {
        operation_error(
            sequence.id.as_str(),
            &format!("sequence relation graph is invalid: {errors}"),
        )
    })?;
    project.sequences.insert(index, sequence.clone());
    project.relations.extend_from_slice(relations);
    changed_tree::sequence(sequence, changed);
    for relation in relations {
        changed.relation(relation.id.clone());
    }
    changed.project(project.id.clone());
    Ok(())
}

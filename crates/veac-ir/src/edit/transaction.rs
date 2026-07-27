use sha2::{Digest, Sha256};

use crate::*;

use super::{apply_operation, check_preconditions, diagnostic, operation_error, ChangeSet};

pub fn apply_edit_batch(project: &ProjectEnvelope, batch: &EditBatch) -> EditOutcome {
    let batch_json = serde_json::to_value(batch).ok();
    if !batch.atomic
        || batch.operations.is_empty()
        || !batch.operation_id.is_valid()
        || !crate::time::safe_u64(batch.base_revision)
        || batch_json
            .as_ref()
            .is_none_or(|value| !crate::validation::json_value_is_ijson(value))
    {
        return rejected(
            project,
            diagnostic(
                "INVALID_EDIT_BATCH",
                batch.operation_id.as_str(),
                "/",
                "edit batches must be atomic, non-empty, and use exact I-JSON values",
            ),
        );
    }
    if let Err(errors) = validate(project) {
        return EditOutcome::Rejected {
            current_revision: project.project.revision,
            diagnostics: errors.into_diagnostics(),
        };
    }
    let request_hash = match edit_batch_hash(batch) {
        Ok(hash) => hash,
        Err(error) => return rejected(project, error),
    };
    if let Some(applied) = project
        .project
        .applied_operations
        .iter()
        .find(|applied| applied.id == batch.operation_id)
    {
        return if applied.request_hash == request_hash {
            EditOutcome::NoChange {
                project: project.clone(),
                current_revision: project.project.revision,
                operation_recorded: true,
            }
        } else {
            EditOutcome::Conflict {
                current_revision: project.project.revision,
                diagnostics: vec![diagnostic(
                    "OPERATION_ID_REUSE",
                    batch.operation_id.as_str(),
                    "/operation_id",
                    "operation ID was already used for different batch content",
                )],
            }
        };
    }
    if batch.base_revision != project.project.revision {
        return EditOutcome::Conflict {
            current_revision: project.project.revision,
            diagnostics: vec![diagnostic(
                "STALE_REVISION",
                batch.operation_id.as_str(),
                "/base_revision",
                "edit batch was based on a stale project revision",
            )],
        };
    }
    if let Err(error) = check_preconditions(&project.project, &batch.preconditions) {
        return EditOutcome::Conflict {
            current_revision: project.project.revision,
            diagnostics: vec![error],
        };
    }
    apply_valid_batch(project, batch, request_hash)
}

fn apply_valid_batch(
    project: &ProjectEnvelope,
    batch: &EditBatch,
    request_hash: String,
) -> EditOutcome {
    let mut next = project.clone();
    let mut changed = ChangeSet::new();
    for operation in &batch.operations {
        if let Err(error) = apply_operation(&mut next.project, operation, &mut changed) {
            return rejected(project, error);
        }
    }
    let semantic_changed = next.project != project.project;
    let Some(new_revision) = next
        .project
        .revision
        .checked_add(1)
        .filter(|value| crate::time::safe_u64(*value))
    else {
        return rejected(
            project,
            diagnostic(
                "REVISION_OVERFLOW",
                batch.operation_id.as_str(),
                "/project/revision",
                "project revision cannot be incremented",
            ),
        );
    };
    next.project.revision = new_revision;
    next.project.applied_operations.push(AppliedOperation {
        id: batch.operation_id.clone(),
        request_hash,
    });
    if let Err(errors) = validate(&next) {
        return EditOutcome::Rejected {
            current_revision: project.project.revision,
            diagnostics: errors.into_diagnostics(),
        };
    }
    if semantic_changed {
        EditOutcome::Applied {
            project: next,
            new_revision,
            changed_objects: changed.into_iter().collect(),
            normalized_operations: batch.operations.clone(),
        }
    } else {
        EditOutcome::NoChange {
            project: next,
            current_revision: new_revision,
            operation_recorded: true,
        }
    }
}

fn edit_batch_hash(batch: &EditBatch) -> Result<String, Diagnostic> {
    serde_json_canonicalizer::to_vec(batch)
        .map(|bytes| crate::canonical::hex_digest(Sha256::digest(bytes)))
        .map_err(|_| {
            operation_error(
                batch.operation_id.as_str(),
                "edit batch contains a value that cannot be canonicalized",
            )
        })
}

fn rejected(project: &ProjectEnvelope, error: Diagnostic) -> EditOutcome {
    EditOutcome::Rejected {
        current_revision: project.project.revision,
        diagnostics: vec![error],
    }
}

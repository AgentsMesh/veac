use std::path::{Path, PathBuf};

use veac_ir::{EditOutcome, ProjectEnvelope};

use crate::error::{CliError, CliResult};

pub(crate) fn run(project: &Path, batch: &Path, output: Option<&Path>, dry_run: bool) -> CliResult {
    let loaded = crate::canonical::load_local(project)?;
    let batch_file = crate::fs::canonical_file(batch, "edit batch")?;
    let batch_json = crate::fs::read_utf8(&batch_file, "edit batch")?;
    let batch = veac_ir::decode_edit_batch_json(&batch_json)
        .map_err(|error| CliError::new("EDIT_BATCH_JSON", error.to_string()))?;
    let outcome = veac_ir::apply_edit_batch(&loaded.envelope, &batch);
    let requested = output.unwrap_or(&loaded.project_file);
    let destination = edit_destination(
        requested,
        &loaded.envelope,
        accepted_project(&outcome),
        &loaded.project_file,
        &batch_file,
    )?;
    if let Some(next) = accepted_project(&outcome) {
        if !dry_run {
            let mut json = match veac_ir::canonical_json(next) {
                Ok(json) => json,
                Err(error) => return Err(CliError::new("CANONICAL_ENCODE", error.to_string())),
            };
            json.push('\n');
            crate::fs::atomic_write(&destination, &json)?;
        }
    }
    emit_outcome(&outcome)?;
    match &outcome {
        EditOutcome::Conflict { diagnostics, .. } | EditOutcome::Rejected { diagnostics, .. } => {
            Err(crate::diagnostic::ir(diagnostics))
        }
        EditOutcome::Applied { .. } | EditOutcome::NoChange { .. } => Ok(()),
    }
}

fn edit_destination(
    requested: &Path,
    current: &ProjectEnvelope,
    next: Option<&ProjectEnvelope>,
    project_file: &Path,
    batch_file: &Path,
) -> CliResult<PathBuf> {
    let mut protected = vec![batch_file.to_path_buf()];
    protected.extend(crate::canonical::local_material_paths(
        current,
        project_file,
    ));
    if let Some(next) = next {
        protected.extend(crate::canonical::local_material_paths(next, requested));
    }
    crate::output::guarded_write_many(requested, protected.iter().map(PathBuf::as_path))
}

fn accepted_project(outcome: &EditOutcome) -> Option<&ProjectEnvelope> {
    match outcome {
        EditOutcome::Applied { project, .. } | EditOutcome::NoChange { project, .. } => {
            Some(project)
        }
        EditOutcome::Conflict { .. } | EditOutcome::Rejected { .. } => None,
    }
}

fn emit_outcome(outcome: &EditOutcome) -> CliResult {
    let mut json = match veac_ir::canonical_edit_outcome_json(outcome) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("EDIT_OUTCOME_ENCODE", error.to_string())),
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

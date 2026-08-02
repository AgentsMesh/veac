use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::{CliError, CliResult};

#[derive(Serialize)]
struct SourceEditReport<'a> {
    module: &'a str,
    previous_revision: &'a veac_lang::source_edit::SourceRevision,
    new_revision: &'a veac_lang::source_edit::SourceRevision,
    destination: Option<String>,
    dry_run: bool,
}

pub(crate) fn run(source: &Path, batch: &Path, output: Option<&Path>, dry_run: bool) -> CliResult {
    let batch_file = crate::fs::canonical_file(batch, "source edit batch")?;
    let json = crate::fs::read_utf8(&batch_file, "source edit batch")?;
    let batch = veac_lang::source_edit::decode_source_edit_batch_json(&json)
        .map_err(|error| CliError::new("SOURCE_EDIT_BATCH_JSON", error.to_string()))?;
    let (root, preview) = veac_lang::program::apply_source_edit_path_with_root(source, &batch)
        .map_err(|error| transaction_error(source, error))?;
    let entry = root.join(preview.compiled.root_module());
    let target = module_path(&entry, &preview.module)?;
    let envelope =
        veac_lang::authoring::lower_document(preview.compiled.document()).map_err(|errors| {
            crate::diagnostic::authoring(&entry, preview.compiled.expanded_source(), errors)
        })?;
    let mut protected = preview
        .compiled
        .sources()
        .keys()
        .map(|module| root.join(module))
        .collect::<Vec<_>>();
    protected.push(batch_file);
    protected.extend(crate::canonical::local_material_paths(&envelope, &entry));
    protected.push(root.join(crate::fs::SOURCE_LOCK_NAME));
    let destination = destination(output, &target, protected)?;
    let aliases_target = crate::output::same_existing_or_equal(&destination, &target)?;
    let in_place = output.is_none() || destination == target;
    if output.is_some() && aliases_target && !in_place {
        return Err(CliError::new(
            "OUTPUT_OVERWRITES_INPUT",
            format!("output {} aliases the edited source", destination.display()),
        ));
    }
    if dry_run {
        revalidate_graph(&root, &preview)?;
    } else if in_place {
        commit_in_place(&root, &preview)?;
    } else {
        revalidate_graph(&root, &preview)?;
        crate::fs::atomic_write(&destination, preview.source())?;
    }
    emit_report(
        &preview,
        (!dry_run).then(|| destination.display().to_string()),
        dry_run,
    )
}

fn module_path(entry: &Path, module: &str) -> CliResult<PathBuf> {
    veac_lang::source_edit::validate_module_path(module)
        .map_err(|error| CliError::new("SOURCE_EDIT_MODULE", error.to_string()))?;
    let root = entry.parent().unwrap_or_else(|| Path::new("."));
    Ok(root.join(module))
}

fn revalidate_graph(root: &Path, preview: &veac_lang::program::SourceEditPreview) -> CliResult {
    crate::fs::ensure_source_graph_unchanged(
        root,
        preview.compiled.root_module(),
        preview.previous_modules(),
        &preview.previous_revision,
    )
}

fn commit_in_place(root: &Path, preview: &veac_lang::program::SourceEditPreview) -> CliResult {
    let source_lock = crate::fs::SourceGraphLock::acquire(root)?;
    source_lock.revalidate(root)?;
    revalidate_graph(root, preview)?;
    source_lock.revalidate(root)?;
    source_lock.commit_module(
        root,
        &preview.module,
        preview.previous_source(),
        preview.source(),
    )
}

fn destination(
    output: Option<&Path>,
    target: &Path,
    protected: Vec<PathBuf>,
) -> CliResult<PathBuf> {
    let Some(output) = output else {
        return Ok(target.to_path_buf());
    };
    let protected = protected
        .into_iter()
        .filter(|path| path != target)
        .collect::<Vec<_>>();
    crate::output::guarded_write_portable(output, protected.iter().map(PathBuf::as_path))
}

fn transaction_error(entry: &Path, error: veac_lang::program::SourceTransactionError) -> CliError {
    match error {
        veac_lang::program::SourceTransactionError::Program(errors) => {
            crate::diagnostic::program(entry, errors)
        }
        other => CliError::new("SOURCE_EDIT_REJECTED", other.to_string()),
    }
}

fn emit_report(
    preview: &veac_lang::program::SourceEditPreview,
    destination: Option<String>,
    dry_run: bool,
) -> CliResult {
    let report = SourceEditReport {
        module: &preview.module,
        previous_revision: &preview.previous_revision,
        new_revision: &preview.new_revision,
        destination,
        dry_run,
    };
    let mut json = serde_json::to_string_pretty(&report)
        .map_err(|error| CliError::new("SOURCE_EDIT_REPORT_JSON", error.to_string()))?;
    json.push('\n');
    crate::fs::write_stdout(&json)
}

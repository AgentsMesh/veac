use std::path::Path;

use serde::Serialize;

use crate::error::{CliError, CliResult};
use veac_lang::program::ExecutableSourceEditPreview as EditPreview;

mod output;

#[derive(Serialize)]
struct SourceEditReport<'a> {
    modules: Vec<&'a str>,
    previous_revision: &'a veac_lang::source_edit::SourceRevision,
    new_revision: &'a veac_lang::source_edit::SourceRevision,
    destinations: Vec<String>,
    dry_run: bool,
}

pub(crate) fn run(
    source: &Path,
    batch: &Path,
    inputs: Option<&Path>,
    inline_inputs: &[String],
    output: Option<&Path>,
    dry_run: bool,
) -> CliResult {
    let batch_file = crate::fs::canonical_file(batch, "source edit batch")?;
    let json = crate::fs::read_utf8(&batch_file, "source edit batch")?;
    let batch = veac_lang::source_edit::decode_source_edit_batch_json(&json)
        .map_err(|error| CliError::new("SOURCE_EDIT_BATCH_JSON", error.to_string()))?;
    let (root, preview) = crate::frontend::read_source_graph(source, |location| {
        let (_, candidate) = veac_lang::program::prepare_executable_source_edit_path_with_root(
            location.path(),
            &batch,
        )
        .map_err(|error| transaction_error(source, error))?;
        let inputs = crate::frontend::build_inputs(inputs, inline_inputs, candidate.program())?;
        candidate
            .execute(&inputs)
            .map_err(|error| transaction_error(source, error))
    })?;
    let entry = root.join(preview.built.root_module());
    let destinations = output::publish(
        &root,
        &entry,
        &batch_file,
        inputs,
        &preview,
        output,
        dry_run,
    )?;
    emit_report(&preview, destinations, dry_run)
}

fn transaction_error(entry: &Path, error: veac_lang::program::SourceTransactionError) -> CliError {
    match error {
        veac_lang::program::SourceTransactionError::Program(errors) => {
            crate::diagnostic::program(entry, errors)
        }
        other => CliError::new("SOURCE_EDIT_REJECTED", other.to_string()),
    }
}

fn emit_report(preview: &EditPreview, destinations: Vec<String>, dry_run: bool) -> CliResult {
    let report = SourceEditReport {
        modules: preview.changed_modules(),
        previous_revision: &preview.previous_revision,
        new_revision: &preview.new_revision,
        destinations,
        dry_run,
    };
    let mut json = serde_json::to_string_pretty(&report)
        .map_err(|error| CliError::new("SOURCE_EDIT_REPORT_JSON", error.to_string()))?;
    json.push('\n');
    crate::fs::write_stdout(&json)
}

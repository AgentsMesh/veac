use std::path::Path;

use crate::error::{CliError, CliResult};

pub(crate) fn run(source: &Path) -> CliResult {
    let program = veac_lang::program::compile_path(source)
        .map_err(|errors| crate::diagnostic::program(source, errors))?;
    let inventory = program
        .source_index()
        .map_err(|errors| crate::diagnostic::program(source, errors))?
        .inventory();
    let mut json = serde_json::to_string_pretty(&inventory)
        .map_err(|error| CliError::new("SOURCE_INDEX_JSON", error.to_string()))?;
    json.push('\n');
    crate::fs::write_stdout(&json)
}

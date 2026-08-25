use std::path::Path;
use std::path::PathBuf;

use crate::error::{CliError, CliResult};

pub(crate) fn run(source: &Path, package_roots: &[PathBuf]) -> CliResult {
    let program = crate::frontend::prepare_source_graph(source, package_roots)?;
    let inventory = match program.source_inventory() {
        Ok(inventory) => inventory,
        Err(errors) => return Err(crate::diagnostic::program(source, errors)),
    };
    let mut json = match serde_json::to_string_pretty(&inventory) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("SOURCE_INDEX_JSON", error.to_string())),
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

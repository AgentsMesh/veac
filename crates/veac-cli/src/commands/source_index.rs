use std::path::Path;

use crate::error::{CliError, CliResult};

pub(crate) fn run(source: &Path) -> CliResult {
    let program = crate::frontend::prepare_source_graph(source)?;
    let inventory = program
        .source_inventory()
        .map_err(|errors| crate::diagnostic::program(source, errors))?;
    let mut json = serde_json::to_string_pretty(&inventory)
        .map_err(|error| CliError::new("SOURCE_INDEX_JSON", error.to_string()))?;
    json.push('\n');
    crate::fs::write_stdout(&json)
}

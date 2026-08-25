use std::path::Path;
use std::path::PathBuf;

use crate::error::CliResult;

pub(crate) fn run(source: &Path, package_roots: &[PathBuf]) -> CliResult {
    let revision = crate::frontend::source_graph_revision(source, package_roots)?;
    let mut json = match serde_json::to_string_pretty(&revision) {
        Ok(json) => json,
        Err(error) => {
            return Err(crate::error::CliError::new(
                "SOURCE_REVISION_JSON",
                error.to_string(),
            ))
        }
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

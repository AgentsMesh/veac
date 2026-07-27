use std::path::Path;

use crate::environment::Environment;
use crate::error::{CliError, CliResult};

pub(crate) fn run(file: &Path, environment: &dyn Environment) -> CliResult {
    let file = crate::fs::canonical_file(file, "media")?;
    let snapshot = environment.probe(&file, veac_runtime::asset::auto_stream_intent())?;
    let mut json = match serde_json::to_string_pretty(&snapshot) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("PROBE_ENCODE", error.to_string())),
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

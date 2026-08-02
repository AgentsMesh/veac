use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;
use veac_runtime::workflow::{WorkflowErrorKind, WorkflowResult};

use crate::{CliError, CliResult};

pub(crate) fn read_json<T: DeserializeOwned>(path: &Path, role: &str) -> CliResult<T> {
    let input = crate::fs::read_utf8(path, role)?;
    parse_json(&input, role)
}

pub(crate) fn read_json_bounded<T: DeserializeOwned>(
    path: &Path,
    role: &str,
    max_bytes: u64,
) -> CliResult<T> {
    let input = crate::fs::read_utf8_bounded(path, role, max_bytes)?;
    parse_json(&input, role)
}

fn parse_json<T: DeserializeOwned>(input: &str, role: &str) -> CliResult<T> {
    if let Err(error) = veac_ir::reject_duplicate_json_keys(input) {
        return Err(CliError::new("WORKFLOW_JSON", error.to_string()));
    }
    match serde_json::from_str(input) {
        Ok(value) => Ok(value),
        Err(error) => Err(CliError::new(
            "WORKFLOW_JSON",
            format!("invalid {role}: {error}"),
        )),
    }
}

pub(crate) fn canonical<T: Serialize>(value: &T) -> CliResult<Vec<u8>> {
    result(serde_json_canonicalizer::to_vec(value), "WORKFLOW_JSON")
}

pub(crate) fn result<T, E: std::fmt::Display>(value: Result<T, E>, code: &str) -> CliResult<T> {
    match value {
        Ok(value) => Ok(value),
        Err(error) => Err(CliError::new(code, error.to_string())),
    }
}

pub(crate) fn workflow<T>(value: WorkflowResult<T>, code: &str) -> CliResult<T> {
    value.map_err(|error| {
        let resource_limit = error.kind == WorkflowErrorKind::ResourceLimit;
        let message = error_chain(&error);
        if resource_limit {
            CliError::resource_limit(code, message)
        } else {
            CliError::new(code, message)
        }
    })
}

fn error_chain(error: &(dyn std::error::Error + 'static)) -> String {
    let mut message = error.to_string();
    let mut source = error.source();
    while let Some(error) = source {
        message.push_str(": ");
        message.push_str(&error.to_string());
        source = error.source();
    }
    message
}

pub(crate) fn write(bytes: &[u8], output: Option<&Path>, protected: &[PathBuf]) -> CliResult {
    let mut content = String::from_utf8(bytes.to_vec())
        .map_err(|error| CliError::new("WORKFLOW_JSON", error.to_string()))?;
    content.push('\n');
    match output {
        None => crate::fs::write_stdout(&content),
        Some(path) if path == Path::new("-") => crate::fs::write_stdout(&content),
        Some(path) => {
            let destination =
                crate::output::guarded_write_many(path, protected.iter().map(PathBuf::as_path))?;
            crate::fs::atomic_write(&destination, &content)
        }
    }
}

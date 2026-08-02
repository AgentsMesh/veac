use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::error::{CliError, CliResult};

pub(super) fn read_json<T: DeserializeOwned>(path: &Path, role: &str) -> CliResult<T> {
    let input = crate::fs::read_utf8(path, role)?;
    if let Err(error) = veac_ir::reject_duplicate_json_keys(&input) {
        return Err(CliError::new("OTIO_JSON", error.to_string()));
    }
    match serde_json::from_str(&input) {
        Ok(value) => Ok(value),
        Err(error) => Err(CliError::new("OTIO_JSON", error.to_string())),
    }
}

pub(super) fn error(error: veac_otio::OtioError) -> CliError {
    CliError::new("OTIO_CONTRACT", error.to_string())
}

pub(super) fn require_loss(
    report: &veac_otio::OtioLossReport,
    allow_lossy: bool,
    loss_path: Option<&Path>,
) -> CliResult {
    if report.is_empty() {
        return Ok(());
    }
    if allow_lossy && loss_path.is_some() {
        return Ok(());
    }
    let first = &report.losses[0];
    Err(CliError::new(
        "OTIO_LOSS_UNACKNOWLEDGED",
        format!(
            "conversion has {} loss(es); first is {}: {}; use --allow-lossy with --loss-report",
            report.len(),
            first.field,
            first.reason
        ),
    ))
}

pub(super) fn write_outputs(
    content: &str,
    output: Option<&Path>,
    loss: Option<(&str, &Path)>,
    protected: &[PathBuf],
) -> CliResult {
    let main = destination(output, protected)?;
    let mut report_protected = protected.to_vec();
    if let Some(path) = &main {
        report_protected.push(path.clone());
    }
    let report = loss
        .map(|(_, path)| {
            if path == Path::new("-") {
                return Err(CliError::new(
                    "OTIO_LOSS_PATH",
                    "loss report must be a file, not stdout",
                ));
            }
            crate::output::guarded_write_many(path, report_protected.iter().map(PathBuf::as_path))
        })
        .transpose()?;
    if let (Some((value, _)), Some(path)) = (loss, report) {
        crate::fs::atomic_write(&path, &newline(value))?;
    }
    match main {
        Some(path) => crate::fs::atomic_write(&path, &newline(content)),
        None => crate::fs::write_stdout(&newline(content)),
    }
}

fn destination(output: Option<&Path>, protected: &[PathBuf]) -> CliResult<Option<PathBuf>> {
    match output {
        None => Ok(None),
        Some(path) if path == Path::new("-") => Ok(None),
        Some(path) => {
            crate::output::guarded_write_many(path, protected.iter().map(PathBuf::as_path))
                .map(Some)
        }
    }
}

fn newline(value: &str) -> String {
    if value.ends_with('\n') {
        value.to_owned()
    } else {
        format!("{value}\n")
    }
}

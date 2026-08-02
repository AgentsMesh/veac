use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;

use crate::error::{CliError, CliResult};

pub(super) fn read_json<T: DeserializeOwned>(path: &Path, role: &str) -> CliResult<T> {
    let input = crate::fs::read_utf8(path, role)?;
    if let Err(error) = veac_ir::reject_duplicate_json_keys(&input) {
        return Err(CliError::new("CAPTION_JSON", error.to_string()));
    }
    match serde_json::from_str(&input) {
        Ok(value) => Ok(value),
        Err(error) => Err(CliError::new("CAPTION_JSON", error.to_string())),
    }
}

pub(super) fn caption_error(error: veac_caption::CaptionError) -> CliError {
    CliError::new("CAPTION_CONTRACT", error.to_string())
}

pub(super) fn require_acknowledged_loss(
    report: &veac_caption::LossReport,
    allow_lossy: bool,
) -> CliResult {
    if report.is_empty() || allow_lossy {
        return Ok(());
    }
    let first = &report.losses[0];
    Err(CliError::new(
        "CAPTION_LOSS_UNACKNOWLEDGED",
        format!(
            "conversion would lose {} field(s); first loss is {}: {}",
            report.losses.len(),
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
    let report = match loss {
        Some((_, path)) if path == Path::new("-") => {
            return Err(CliError::new(
                "CAPTION_LOSS_PATH",
                "loss report must be a file, not stdout",
            ));
        }
        Some((_, path)) => Some(crate::output::guarded_write_many(
            path,
            report_protected.iter().map(PathBuf::as_path),
        )?),
        None => None,
    };
    if let (Some((report_content, _)), Some(path)) = (loss, report) {
        crate::fs::atomic_write(&path, &with_newline(report_content))?;
    }
    match main {
        Some(path) => crate::fs::atomic_write(&path, &with_newline(content)),
        None => crate::fs::write_stdout(&with_newline(content)),
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

fn with_newline(value: &str) -> String {
    let mut output = value.to_owned();
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

use std::fs::File;
use std::io::Read;
use std::path::Path;

use crate::{BuildError, BuildResult, CancellationToken, ExecutionErrorKind};

pub(super) fn matches(
    destination: &Path,
    expected_archive: &Path,
    parent: &Path,
    cancellation: &CancellationToken,
) -> BuildResult<bool> {
    let temporary = tempfile::Builder::new()
        .prefix(".veac-directory-compare-")
        .tempdir_in(parent)
        .map_err(io)?;
    let archive = temporary.path().join("existing.artifact");
    super::pack(destination, &archive, cancellation).map_err(|error| match error.kind() {
        ExecutionErrorKind::Cancelled => BuildError::cancelled(error.message()),
        ExecutionErrorKind::Failed => BuildError::invalid(error.message()),
    })?;
    equal(&archive, expected_archive, cancellation)
}

fn equal(left: &Path, right: &Path, cancellation: &CancellationToken) -> BuildResult<bool> {
    if std::fs::metadata(left).map_err(io)?.len() != std::fs::metadata(right).map_err(io)?.len() {
        return Ok(false);
    }
    let mut left = File::open(left).map_err(io)?;
    let mut right = File::open(right).map_err(io)?;
    let mut left_buffer = [0_u8; 64 * 1024];
    let mut right_buffer = [0_u8; 64 * 1024];
    loop {
        if cancellation.is_cancelled() {
            return Err(BuildError::cancelled(
                "cancelled while checking an existing directory delivery",
            ));
        }
        let left_read = left.read(&mut left_buffer).map_err(io)?;
        let right_read = right.read(&mut right_buffer).map_err(io)?;
        if left_read != right_read || left_buffer[..left_read] != right_buffer[..right_read] {
            return Ok(false);
        }
        if left_read == 0 {
            return Ok(true);
        }
    }
}

fn io(error: std::io::Error) -> BuildError {
    BuildError::new(
        crate::BuildErrorKind::Cache,
        format!("existing directory delivery check failed: {error}"),
    )
}

#[cfg(test)]
#[path = "existing/tests.rs"]
mod tests;

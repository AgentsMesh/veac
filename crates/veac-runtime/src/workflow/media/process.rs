use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

use veac_artifact::MediaArtifactLimits;

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};
use crate::tool::{spawn_pinned_until, PinnedSpawnError};

const MAX_STDERR_BYTES: u64 = 1024 * 1024;
const POLL_INTERVAL: Duration = Duration::from_millis(20);

pub(super) fn run_while(
    executable: &Path,
    arguments: &[OsString],
    output: &Path,
    limits: MediaArtifactLimits,
    deadline: Instant,
    mut guard: impl FnMut() -> bool,
) -> WorkflowResult<()> {
    if Instant::now() >= deadline {
        return limit("FFmpeg artifact derivation exceeded its wall-clock budget");
    }
    if !guard() {
        return limit("FFmpeg artifact derivation was cancelled");
    }
    let mut stderr = tempfile::tempfile()?;
    let writer = stderr.try_clone()?;
    let mut command = Command::new(executable);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::from(writer));
    crate::process_group::configure(&mut command);
    let mut child = spawn_pinned_until(&mut command, deadline).map_err(launch_error)?;
    let status = loop {
        if !guard() {
            crate::process_group::stop(&mut child);
            return limit("FFmpeg artifact derivation was cancelled");
        }
        let stderr_bytes = match stderr.metadata() {
            Ok(value) => value.len(),
            Err(error) => {
                crate::process_group::stop(&mut child);
                return Err(error.into());
            }
        };
        if stderr_bytes > MAX_STDERR_BYTES {
            crate::process_group::stop(&mut child);
            return limit("FFmpeg stderr exceeded the execution budget");
        }
        let output_bytes = match output_size(output) {
            Ok(value) => value,
            Err(error) => {
                crate::process_group::stop(&mut child);
                return Err(error);
            }
        };
        if output_bytes > limits.max_payload_bytes {
            crate::process_group::stop(&mut child);
            return limit("FFmpeg output exceeded the artifact payload budget");
        }
        if Instant::now() >= deadline {
            crate::process_group::stop(&mut child);
            return limit("FFmpeg artifact derivation exceeded its wall-clock budget");
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                crate::process_group::terminate(&child);
                break status;
            }
            Ok(None) => {}
            Err(error) => {
                crate::process_group::stop(&mut child);
                return Err(error.into());
            }
        }
        std::thread::sleep(POLL_INTERVAL);
    };
    if !status.success() {
        return Err(WorkflowError::new(
            WorkflowErrorKind::ToolFailure,
            format!(
                "FFmpeg artifact derivation failed: {}",
                detail(&mut stderr)?
            ),
        ));
    }
    verify_output(output, limits.max_payload_bytes)
}

fn output_size(path: &Path) -> WorkflowResult<u64> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(0),
        Err(error) => return Err(error.into()),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(WorkflowError::new(
            WorkflowErrorKind::ToolFailure,
            "FFmpeg output is not a regular non-symlink file",
        ));
    }
    Ok(metadata.len())
}

fn verify_output(path: &Path, max_bytes: u64) -> WorkflowResult<()> {
    let size = output_size(path)?;
    if size == 0 {
        return Err(WorkflowError::new(
            WorkflowErrorKind::ToolFailure,
            "FFmpeg did not produce a regular non-empty artifact",
        ));
    }
    if size > max_bytes {
        return limit("FFmpeg output exceeded the artifact payload budget");
    }
    Ok(())
}

fn detail(file: &mut File) -> WorkflowResult<String> {
    file.rewind()?;
    let mut bytes = Vec::new();
    file.take(MAX_STDERR_BYTES + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > MAX_STDERR_BYTES {
        return Ok("stderr exceeded the execution budget".to_owned());
    }
    Ok(String::from_utf8_lossy(&bytes).trim().to_owned())
}

fn start_error(error: std::io::Error) -> WorkflowError {
    WorkflowError::with_source(
        WorkflowErrorKind::ToolFailure,
        "failed to start FFmpeg artifact derivation",
        error,
    )
}

fn launch_error(error: PinnedSpawnError) -> WorkflowError {
    match error {
        PinnedSpawnError::Deadline => WorkflowError::new(
            WorkflowErrorKind::ResourceLimit,
            "FFmpeg artifact derivation exceeded its wall-clock budget",
        ),
        PinnedSpawnError::Io(error) => start_error(error),
    }
}

fn limit<T>(message: &str) -> WorkflowResult<T> {
    Err(WorkflowError::new(
        WorkflowErrorKind::ResourceLimit,
        message,
    ))
}

#[cfg(test)]
#[path = "process/coverage_tests.rs"]
mod coverage_tests;
#[cfg(test)]
#[path = "process/tests.rs"]
mod tests;

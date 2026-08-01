use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use std::process::{Child, Command, Output, Stdio};
use std::time::{Duration, Instant};

use crate::RuntimeError;

mod output;

const POLL_INTERVAL: Duration = Duration::from_millis(20);
pub(super) const MAX_STDOUT_BYTES: u64 = 16 * 1024 * 1024;
pub(super) const MAX_STDERR_BYTES: u64 = veac_artifact::MAX_ARTIFACT_METADATA_BYTES;
pub(super) const MAX_OUTPUT_BYTES: u64 = veac_artifact::MAX_RENDER_TASK_OUTPUT_BYTES;
pub(super) const MAX_WALL_TIME: Duration =
    Duration::from_secs(veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS);

pub(super) struct ProcessLimits<'a> {
    pub deadline: Instant,
    pub max_stdout_bytes: u64,
    pub max_stderr_bytes: u64,
    pub output_root: Option<&'a Path>,
    pub working_directory: Option<&'a Path>,
    pub max_output_bytes: u64,
}

pub(super) fn run(
    binary: &Path,
    arguments: &[String],
    limits: ProcessLimits<'_>,
) -> Result<Output, RuntimeError> {
    validate(&limits)?;
    let mut stdout = tempfile::tempfile().map_err(io_error)?;
    let mut stderr = tempfile::tempfile().map_err(io_error)?;
    let mut command = Command::new(binary);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::from(stdout.try_clone().map_err(io_error)?))
        .stderr(Stdio::from(stderr.try_clone().map_err(io_error)?));
    if let Some(directory) = limits.working_directory {
        command.current_dir(directory);
    }
    crate::process_group::configure(&mut command);
    let mut child = command.spawn().map_err(io_error)?;
    let status = loop {
        let resources = resource_violation(&stdout, &stderr, &limits);
        let violation = match resources {
            Ok(value) => value,
            Err(error) => return stop_with(&mut child, error),
        };
        if let Some(message) = violation {
            return stop_limit(&mut child, message);
        }
        if Instant::now() >= limits.deadline {
            return stop_limit(&mut child, "FFmpeg exceeded its wall-clock limit");
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => std::thread::sleep(POLL_INTERVAL),
            Err(error) => return stop_with(&mut child, io_error(error)),
        }
    };
    crate::process_group::terminate(&child);
    if let Some(message) = resource_violation(&stdout, &stderr, &limits)? {
        return Err(RuntimeError::resource_limit(message));
    }
    Ok(Output {
        status,
        stdout: read(&mut stdout, limits.max_stdout_bytes)?,
        stderr: read(&mut stderr, limits.max_stderr_bytes)?,
    })
}

fn resource_violation(
    stdout: &File,
    stderr: &File,
    limits: &ProcessLimits<'_>,
) -> Result<Option<&'static str>, RuntimeError> {
    if length(stdout)? > limits.max_stdout_bytes {
        return Ok(Some("FFmpeg stdout exceeded its byte limit"));
    }
    if length(stderr)? > limits.max_stderr_bytes {
        return Ok(Some("FFmpeg stderr exceeded its byte limit"));
    }
    if let Some(root) = limits.output_root {
        if output::tree_size(root)? > limits.max_output_bytes {
            return Ok(Some("FFmpeg output exceeded its byte limit"));
        }
    }
    Ok(None)
}

fn validate(limits: &ProcessLimits<'_>) -> Result<(), RuntimeError> {
    let wall_time = limits.deadline.checked_duration_since(Instant::now());
    if wall_time.is_none_or(|value| value.is_zero()) {
        return Err(RuntimeError::resource_limit(
            "FFmpeg exceeded its wall-clock limit",
        ));
    }
    if wall_time.is_some_and(|value| value > MAX_WALL_TIME)
        || limits.max_stdout_bytes == 0
        || limits.max_stdout_bytes > MAX_STDOUT_BYTES
        || limits.max_stderr_bytes == 0
        || limits.max_stderr_bytes > MAX_STDERR_BYTES
        || limits.max_output_bytes == 0
        || limits.max_output_bytes > MAX_OUTPUT_BYTES
    {
        return Err(RuntimeError::new("invalid FFmpeg process resource policy"));
    }
    Ok(())
}

fn length(file: &File) -> Result<u64, RuntimeError> {
    file.metadata().map(|value| value.len()).map_err(io_error)
}

fn read(file: &mut File, max_bytes: u64) -> Result<Vec<u8>, RuntimeError> {
    file.rewind().map_err(io_error)?;
    let mut bytes = Vec::new();
    file.take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(io_error)?;
    if bytes.len() as u64 > max_bytes {
        return Err(RuntimeError::resource_limit(
            "FFmpeg process output exceeded its byte limit",
        ));
    }
    Ok(bytes)
}

fn stop_limit<T>(child: &mut Child, message: &str) -> Result<T, RuntimeError> {
    stop_with(child, RuntimeError::resource_limit(message))
}

fn stop_with<T>(child: &mut Child, error: RuntimeError) -> Result<T, RuntimeError> {
    crate::process_group::stop(child);
    Err(error)
}

fn io_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(error.to_string())
}

#[cfg(test)]
#[path = "runner/coverage_tests.rs"]
mod coverage_tests;
#[cfg(test)]
mod tests;

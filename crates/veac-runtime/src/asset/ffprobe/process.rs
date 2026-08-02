use std::ffi::OsString;
use std::fs::File;
use std::io::{Read, Seek};
use std::path::Path;
use std::process::{Command, ExitStatus, Stdio};
use std::time::{Duration, Instant};

use crate::asset::ProbeError;

const MAX_STDERR_BYTES: u64 = veac_artifact::MAX_ARTIFACT_METADATA_BYTES;
const POLL_INTERVAL: Duration = Duration::from_millis(20);

#[derive(Debug)]
pub(super) struct Output {
    pub(super) status: ExitStatus,
    pub(super) stdout: Vec<u8>,
    pub(super) stderr: Vec<u8>,
}

pub(super) fn run(
    binary: &Path,
    arguments: &[OsString],
    max_stdout_bytes: u64,
    deadline: Instant,
    operation: &'static str,
) -> Result<Output, ProbeError> {
    if Instant::now() >= deadline {
        return Err(ProbeError::ResourceLimit { operation });
    }
    let mut stdout = tempfile::tempfile().map_err(|error| spawn(binary, error))?;
    let mut stderr = tempfile::tempfile().map_err(|error| spawn(binary, error))?;
    let mut command = Command::new(binary);
    command
        .args(arguments)
        .stdin(Stdio::null())
        .stdout(Stdio::from(
            stdout.try_clone().map_err(|error| spawn(binary, error))?,
        ))
        .stderr(Stdio::from(
            stderr.try_clone().map_err(|error| spawn(binary, error))?,
        ));
    crate::process_group::configure(&mut command);
    let mut child = command.spawn().map_err(|error| spawn(binary, error))?;
    let status = loop {
        let output_bytes = match length(&stdout, binary) {
            Ok(value) => value,
            Err(error) => {
                crate::process_group::stop(&mut child);
                return Err(error);
            }
        };
        let error_bytes = match length(&stderr, binary) {
            Ok(value) => value,
            Err(error) => {
                crate::process_group::stop(&mut child);
                return Err(error);
            }
        };
        let over_output = output_bytes > max_stdout_bytes;
        let over_error = error_bytes > MAX_STDERR_BYTES;
        if over_output || over_error || Instant::now() >= deadline {
            crate::process_group::stop(&mut child);
            return Err(ProbeError::ResourceLimit { operation });
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                crate::process_group::terminate(&child);
                break status;
            }
            Ok(None) => {}
            Err(error) => {
                crate::process_group::stop(&mut child);
                return Err(spawn(binary, error));
            }
        }
        std::thread::sleep(POLL_INTERVAL);
    };
    Ok(Output {
        status,
        stdout: read(&mut stdout, max_stdout_bytes, binary, operation)?,
        stderr: read(&mut stderr, MAX_STDERR_BYTES, binary, operation)?,
    })
}

fn length(file: &File, binary: &Path) -> Result<u64, ProbeError> {
    file.metadata()
        .map(|value| value.len())
        .map_err(|error| spawn(binary, error))
}

fn read(
    file: &mut File,
    max_bytes: u64,
    binary: &Path,
    operation: &'static str,
) -> Result<Vec<u8>, ProbeError> {
    file.rewind().map_err(|error| spawn(binary, error))?;
    let mut bytes = Vec::new();
    file.take(max_bytes + 1)
        .read_to_end(&mut bytes)
        .map_err(|error| spawn(binary, error))?;
    if bytes.len() as u64 > max_bytes {
        return Err(ProbeError::ResourceLimit { operation });
    }
    Ok(bytes)
}

fn spawn(binary: &Path, source: std::io::Error) -> ProbeError {
    ProbeError::ProcessSpawn {
        binary: binary.to_string_lossy().into_owned(),
        source,
    }
}

#[cfg(test)]
#[path = "process/tests.rs"]
mod tests;

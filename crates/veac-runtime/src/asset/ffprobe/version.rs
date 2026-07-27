use std::ffi::OsString;
use std::path::Path;
use std::time::Instant;

use super::{process, ProbeError};
use crate::tool::PinnedExecutable;

pub(super) fn read(
    binary: &Path,
    tool: &PinnedExecutable,
    deadline: Instant,
) -> Result<String, ProbeError> {
    let executable = tool
        .launch_until(deadline)
        .map_err(|error| tool_error(binary, error))?;
    let output = process::run(
        executable.path(),
        &[OsString::from("-version")],
        4 * 1024,
        deadline,
        "version query",
    )?;
    if !output.status.success() {
        return Err(ProbeError::VersionFailed {
            binary: binary.to_string_lossy().into_owned(),
            status: output.status.code(),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut words = stdout.lines().next().unwrap_or_default().split_whitespace();
    match (words.next(), words.next(), words.next()) {
        (Some("ffprobe"), Some("version"), Some(version)) if !version.is_empty() => Ok(format!(
            "ffprobe version {version} sha256:{}",
            tool.identity().digest
        )),
        _ => Err(ProbeError::InvalidField {
            field: "engine.version",
            value: stdout.into_owned(),
        }),
    }
}

pub(super) fn tool_error(binary: &Path, error: crate::RuntimeError) -> ProbeError {
    if error.kind == crate::RuntimeErrorKind::ResourceLimit {
        ProbeError::ResourceLimit {
            operation: "tool snapshot",
        }
    } else {
        ProbeError::ProcessSpawn {
            binary: binary.to_string_lossy().into_owned(),
            source: std::io::Error::other(error),
        }
    }
}

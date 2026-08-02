use std::fmt;
use std::io;
use std::path::PathBuf;

use veac_ir::{HashAlgorithm, ProbedStreamType};

/// Failures from hashing, invoking ffprobe, or canonicalizing its output.
#[derive(Debug)]
pub enum ProbeError {
    FileNotFound {
        path: PathBuf,
    },
    Io {
        operation: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    ProcessSpawn {
        binary: String,
        source: io::Error,
    },
    ProcessFailed {
        path: PathBuf,
        status: Option<i32>,
        stderr: String,
    },
    IdentityChanged {
        path: PathBuf,
    },
    VersionFailed {
        binary: String,
        status: Option<i32>,
        stderr: String,
    },
    ResourceLimit {
        operation: &'static str,
    },
    InvalidJson {
        source: serde_json::Error,
    },
    InvalidField {
        field: &'static str,
        value: String,
    },
    StreamSelection {
        media_type: ProbedStreamType,
        global_index: u32,
    },
    UnsupportedHashAlgorithm {
        algorithm: HashAlgorithm,
    },
}

impl fmt::Display for ProbeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FileNotFound { path } => write!(formatter, "file not found: {}", path.display()),
            Self::Io {
                operation,
                path,
                source,
            } => write!(
                formatter,
                "failed to {operation} {}: {source}",
                path.display()
            ),
            Self::ProcessSpawn { binary, source } => {
                write!(formatter, "failed to run {binary}: {source}")
            }
            Self::ProcessFailed {
                path,
                status,
                stderr,
            } => write!(
                formatter,
                "ffprobe failed on {} with status {}: {stderr}",
                path.display(),
                status_text(*status)
            ),
            Self::IdentityChanged { path } => {
                write!(
                    formatter,
                    "media changed while being probed: {}",
                    path.display()
                )
            }
            Self::VersionFailed {
                binary,
                status,
                stderr,
            } => write!(
                formatter,
                "{binary} version query failed with status {}: {stderr}",
                status_text(*status)
            ),
            Self::ResourceLimit { operation } => {
                write!(
                    formatter,
                    "ffprobe {operation} exceeded its resource budget"
                )
            }
            Self::InvalidJson { source } => {
                write!(formatter, "failed to parse ffprobe output: {source}")
            }
            Self::InvalidField { field, value } => {
                write!(formatter, "invalid ffprobe field `{field}`: `{value}`")
            }
            Self::StreamSelection {
                media_type,
                global_index,
            } => write!(
                formatter,
                "stream {global_index} cannot be selected as {media_type:?}"
            ),
            Self::UnsupportedHashAlgorithm { algorithm } => {
                write!(
                    formatter,
                    "unsupported media identity algorithm: {algorithm:?}"
                )
            }
        }
    }
}

fn status_text(status: Option<i32>) -> String {
    match status {
        Some(code) => code.to_string(),
        None => "signal".to_owned(),
    }
}

impl std::error::Error for ProbeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } | Self::ProcessSpawn { source, .. } => Some(source),
            Self::InvalidJson { source } => Some(source),
            _ => None,
        }
    }
}

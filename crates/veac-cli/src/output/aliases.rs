use std::fs::{self, Metadata};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

use unicode_normalization::UnicodeNormalization;

use crate::error::{CliError, CliResult};

#[derive(Clone, Copy)]
struct Policy {
    case_insensitive: bool,
    normalization_insensitive: bool,
}

pub(super) fn conflicts(candidate: &Path, protected: &[PathBuf]) -> CliResult<bool> {
    for path in protected {
        if candidate == path || same_existing(candidate, path)? {
            return Ok(true);
        }
    }
    let parent = candidate.parent().unwrap_or(Path::new("."));
    let peers = protected
        .iter()
        .filter(|path| path.parent().unwrap_or(Path::new(".")) == parent)
        .collect::<Vec<_>>();
    if peers.is_empty() {
        return Ok(false);
    }
    folded_conflict(candidate, &peers, detect(parent)?)
}

pub(super) fn conflicts_portable(candidate: &Path, protected: &[PathBuf]) -> CliResult<bool> {
    for path in protected {
        if candidate == path || same_existing(candidate, path)? {
            return Ok(true);
        }
    }
    let policy = Policy {
        case_insensitive: true,
        normalization_insensitive: true,
    };
    folded_conflict(candidate, &protected.iter().collect::<Vec<_>>(), policy)
}

fn folded_conflict(candidate: &Path, peers: &[&PathBuf], policy: Policy) -> CliResult<bool> {
    let candidate = fold(candidate, policy)?;
    peers
        .iter()
        .map(|path| fold(path, policy))
        .try_fold(false, |found, path| Ok(found || path? == candidate))
}

fn detect(parent: &Path) -> CliResult<Policy> {
    let probe = tempfile::Builder::new()
        .prefix(".veac-alias-probe-")
        .tempdir_in(parent)
        .map_err(alias_error)?;
    Ok(Policy {
        case_insensitive: alias(probe.path(), "VEAC-Case-A", "veac-case-a")?,
        normalization_insensitive: alias(probe.path(), "v\u{e9}ac", "ve\u{301}ac")?,
    })
}

fn alias(parent: &Path, authored: &str, alternate: &str) -> CliResult<bool> {
    fs::write(parent.join(authored), b"probe").map_err(alias_error)?;
    match fs::symlink_metadata(parent.join(alternate)) {
        Ok(metadata) => Ok(metadata.is_file()),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(false),
        Err(error) => Err(alias_error(error)),
    }
}

fn fold(path: &Path, policy: Policy) -> CliResult<PathBuf> {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| {
            CliError::new(
                "OUTPUT_ALIAS_POLICY",
                format!("cannot compare non-UTF-8 output name {}", path.display()),
            )
        })?;
    let name = if policy.case_insensitive {
        name.chars().flat_map(char::to_lowercase).collect()
    } else {
        name.to_owned()
    };
    let name = if policy.normalization_insensitive {
        name.nfc().collect::<String>()
    } else {
        name
    };
    Ok(path.parent().unwrap_or(Path::new(".")).join(name))
}

pub(super) fn same_existing(left: &Path, right: &Path) -> CliResult<bool> {
    let (Some(left), Some(right)) = (metadata(left)?, metadata(right)?) else {
        return Ok(false);
    };
    Ok(same_identity(&left, &right))
}

fn metadata(path: &Path) -> CliResult<Option<Metadata>> {
    match fs::metadata(path) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(alias_error(error)),
    }
}

#[cfg(unix)]
fn same_identity(left: &Metadata, right: &Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    left.dev() == right.dev() && left.ino() == right.ino()
}

#[cfg(windows)]
fn same_identity(left: &Metadata, right: &Metadata) -> bool {
    use std::os::windows::fs::MetadataExt;
    left.volume_serial_number() == right.volume_serial_number()
        && left.file_index() == right.file_index()
}

#[cfg(not(any(unix, windows)))]
fn same_identity(_left: &Metadata, _right: &Metadata) -> bool {
    false
}

fn alias_error(error: std::io::Error) -> CliError {
    CliError::new(
        "OUTPUT_ALIAS_POLICY",
        format!("cannot probe output filesystem alias policy: {error}"),
    )
}

#[cfg(test)]
mod tests;

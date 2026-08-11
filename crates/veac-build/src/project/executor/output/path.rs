use std::{fs::File, io::Read, path::Path};

use sha2::{Digest, Sha256};
use veac_artifact::{ContentDigest, DigestAlgorithm};

use crate::{CancellationToken, ExecutionError};

pub(super) fn checked_file(
    workspace: &Path,
    relative: &Path,
) -> Result<std::path::PathBuf, ExecutionError> {
    checked(workspace, relative, false)
}

pub(super) fn checked_directory(
    workspace: &Path,
    relative: &Path,
) -> Result<std::path::PathBuf, ExecutionError> {
    checked(workspace, relative, true)
}

fn checked(
    workspace: &Path,
    relative: &Path,
    directory: bool,
) -> Result<std::path::PathBuf, ExecutionError> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|part| !matches!(part, std::path::Component::Normal(_)))
    {
        return Err(failed("backend output path is not canonical and relative"));
    }
    let path = workspace.join(relative);
    let root = std::fs::canonicalize(workspace)
        .map_err(|error| failed(format!("cannot resolve backend workspace: {error}")))?;
    reject_symlinks(workspace, relative)?;
    let metadata = std::fs::symlink_metadata(&path)
        .map_err(|error| failed(format!("cannot inspect backend output: {error}")))?;
    let expected_kind = if directory {
        metadata.is_dir()
    } else {
        metadata.is_file()
    };
    if metadata.file_type().is_symlink() || !expected_kind {
        return Err(failed(if directory {
            "backend output must be a non-symlink directory"
        } else {
            "backend output must be a regular non-symlink file"
        }));
    }
    let canonical = std::fs::canonicalize(path)
        .map_err(|error| failed(format!("cannot resolve backend output: {error}")))?;
    if !canonical.starts_with(root) {
        return Err(failed("backend output escaped its workspace"));
    }
    Ok(canonical)
}

fn reject_symlinks(workspace: &Path, relative: &Path) -> Result<(), ExecutionError> {
    let mut current = workspace.to_owned();
    for component in relative.components() {
        current.push(component.as_os_str());
        let metadata = std::fs::symlink_metadata(&current)
            .map_err(|error| failed(format!("cannot inspect backend output component: {error}")))?;
        if metadata.file_type().is_symlink() {
            return Err(failed("backend output path contains a symlink"));
        }
    }
    Ok(())
}

pub(super) fn fingerprint(
    path: &Path,
    cancellation: &CancellationToken,
) -> Result<(ContentDigest, u64), ExecutionError> {
    let mut file =
        File::open(path).map_err(|error| failed(format!("cannot open backend output: {error}")))?;
    let mut hash = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        if cancellation.is_cancelled() {
            return Err(ExecutionError::cancelled(
                "cancelled while hashing backend output",
            ));
        }
        let read = file
            .read(&mut buffer)
            .map_err(|error| failed(format!("cannot read backend output: {error}")))?;
        if read == 0 {
            break;
        }
        size = size
            .checked_add(read as u64)
            .ok_or_else(|| failed("backend output size overflow"))?;
        hash.update(&buffer[..read]);
    }
    Ok((
        ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: format!("{:x}", hash.finalize()),
        },
        size,
    ))
}

fn failed(message: impl Into<String>) -> ExecutionError {
    ExecutionError::failed(message)
}

#[cfg(test)]
#[path = "path/coverage_tests.rs"]
mod coverage_tests;

#[cfg(test)]
#[path = "path/tests.rs"]
mod tests;

use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use tempfile::TempDir;
use veac_ir::MediaIdentity;

use crate::RuntimeError;

mod cache;
mod interpreter;
mod permissions;
mod snapshot;
mod spawn;

pub(crate) use cache::{DeadlineCache, DeadlineCacheError};
use permissions::writable_directory;
pub(crate) use spawn::{spawn_pinned_until, PinnedSpawnError};

#[derive(Debug)]
pub(crate) struct PinnedExecutable {
    directory: TempDir,
    path: PathBuf,
    identity: MediaIdentity,
}

#[derive(Debug)]
pub(crate) struct LaunchExecutable {
    _directory: TempDir,
    path: PathBuf,
}

impl PinnedExecutable {
    pub(crate) fn identity(&self) -> &MediaIdentity {
        &self.identity
    }
}

impl Drop for PinnedExecutable {
    fn drop(&mut self) {
        writable_directory(self.directory.path());
    }
}

impl LaunchExecutable {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

fn resolve(binary: &Path) -> Result<PathBuf, RuntimeError> {
    if binary.is_absolute() || binary.components().count() > 1 {
        return canonical(binary);
    }
    let search = match std::env::var_os("PATH") {
        Some(search) => search,
        None => {
            return Err(RuntimeError::new(
                "cannot resolve executable because PATH is unset",
            ))
        }
    };
    resolve_in(binary, &search)
}

fn resolve_in(binary: &Path, search: &OsStr) -> Result<PathBuf, RuntimeError> {
    for directory in std::env::split_paths(search) {
        let candidate = directory.join(binary);
        match std::fs::symlink_metadata(&candidate) {
            Ok(_) => return canonical(&candidate),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(tool_error("inspect PATH executable", error)),
        }
    }
    Err(RuntimeError::new(format!(
        "cannot resolve executable {} on PATH",
        binary.display()
    )))
}

fn canonical(path: &Path) -> Result<PathBuf, RuntimeError> {
    match std::fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(error) => Err(RuntimeError::new(format!(
            "cannot resolve executable {}: {error}",
            path.display()
        ))),
    }
}

fn executable_source(path: &Path) -> Result<(), RuntimeError> {
    let metadata = match std::fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) => return Err(tool_error("inspect executable target", error)),
    };
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(RuntimeError::new(
            "resolved executable must be a regular non-symlink file",
        ));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            return Err(RuntimeError::new("resolved tool is not executable"));
        }
    }
    Ok(())
}

fn tool_error(operation: &str, error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!("cannot {operation}: {error}"))
}

fn io_result(result: std::io::Result<()>, operation: &str) -> Result<(), RuntimeError> {
    match result {
        Ok(()) => Ok(()),
        Err(error) => Err(tool_error(operation, error)),
    }
}

#[cfg(test)]
mod tests;

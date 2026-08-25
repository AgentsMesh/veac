use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use veac_artifact::{ArtifactCommitState, ContentDigest, OwnedStagedFile};

use crate::error::{CliError, CliResult};

mod source_graph;
mod source_location;
mod source_lock;

pub(crate) use source_graph::ensure_source_modules_unchanged;
pub(crate) use source_location::SourceLocation;
pub(crate) use source_lock::SOURCE_LOCK_NAME;
pub(crate) use source_lock::{
    SourceGraphLock, SourceGraphReadLock, SourceModuleGuard, SourceModuleReplacement,
};

pub(crate) fn read_utf8(path: &Path, role: &str) -> CliResult<String> {
    read_utf8_bounded(path, role, veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES)
}

pub(crate) fn read_utf8_bounded(path: &Path, role: &str, max_bytes: u64) -> CliResult<String> {
    let bytes =
        veac_artifact::read_verified_source_bounded(path, None, max_bytes).map_err(|error| {
            CliError::new(
                "READ_FAILED",
                format!("failed to read {role} {}: {error}", path.display()),
            )
        })?;
    String::from_utf8(bytes).map_err(|error| {
        CliError::new(
            "READ_FAILED",
            format!("failed to read {role} {} as UTF-8: {error}", path.display()),
        )
    })
}

pub(crate) fn write_stdout(value: &str) -> CliResult {
    let mut stdout = io::stdout().lock();
    if let Err(error) = stdout
        .write_all(value.as_bytes())
        .and_then(|_| stdout.flush())
    {
        return Err(CliError::new("STDOUT_FAILED", error.to_string()));
    }
    Ok(())
}

pub(crate) fn atomic_write(path: &Path, value: &str) -> CliResult {
    atomic_write_impl(path, value, |_| {})
}

fn atomic_write_impl(path: &Path, value: &str, after_open: impl FnOnce(&Path)) -> CliResult {
    let parent = path.parent().filter(|value| !value.as_os_str().is_empty());
    let parent = parent.unwrap_or(Path::new("."));
    path.file_name().ok_or_else(|| {
        CliError::new(
            "INVALID_OUTPUT_PATH",
            format!("{} has no file name", path.display()),
        )
    })?;
    let mut staged = OwnedStagedFile::new_in(parent)
        .map_err(|error| stage_error("create temporary output", path, error))?;
    after_open(staged.path());
    let result = write_staged(&mut staged, path, value);
    if let Err(error) = result {
        return match staged.discard() {
            Ok(()) => Err(error),
            Err(cleanup) => Err(CliError::new(
                "WRITE_FAILED",
                format!(
                    "{error}; temporary output cleanup also failed for {}: {cleanup}",
                    path.display()
                ),
            )),
        };
    }
    staged
        .persist_replace(path)
        .map_err(|error| stage_error("replace output", path, error))?;
    Ok(())
}

fn write_staged(staged: &mut OwnedStagedFile, destination: &Path, value: &str) -> CliResult {
    if let Err(error) = staged
        .file_mut()
        .write_all(value.as_bytes())
        .and_then(|_| staged.file_mut().sync_all())
    {
        return Err(io_error("write temporary output", staged.path(), error));
    }
    if let Ok(metadata) = fs::metadata(destination) {
        if let Err(error) = staged.file_mut().set_permissions(metadata.permissions()) {
            return Err(io_error(
                "preserve output permissions",
                staged.path(),
                error,
            ));
        }
    }
    let sealed = staged
        .seal()
        .map_err(|error| stage_error("seal temporary output", destination, error))?;
    let expected = ContentDigest::sha256(value.as_bytes());
    if sealed.sha256 != expected.value || sealed.size_bytes != value.len() as u64 {
        return Err(CliError::new(
            "WRITE_FAILED",
            format!(
                "temporary output content changed before replacing {}",
                destination.display()
            ),
        ));
    }
    Ok(())
}

fn io_error(operation: &str, path: &Path, error: io::Error) -> CliError {
    CliError::new(
        "WRITE_FAILED",
        format!("failed to {operation} {}: {error}", path.display()),
    )
}

fn stage_error(operation: &str, path: &Path, error: veac_artifact::ArtifactError) -> CliError {
    let code = match error.commit_state {
        ArtifactCommitState::NotCommitted => "WRITE_FAILED",
        ArtifactCommitState::Committed => "WRITE_COMMIT_UNCERTAIN",
    };
    CliError::new(
        code,
        format!("failed to {operation} {}: {error}", path.display()),
    )
}

pub(crate) fn canonical_file(path: &Path, role: &str) -> CliResult<PathBuf> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        CliError::new(
            "PATH_UNAVAILABLE",
            format!("cannot resolve {role} {}: {error}", path.display()),
        )
    })?;
    if !canonical.is_file() {
        return Err(CliError::new(
            "NOT_A_FILE",
            format!("{role} {} is not a regular file", path.display()),
        ));
    }
    Ok(canonical)
}

pub(crate) fn canonical_directory(path: &Path, role: &str) -> CliResult<PathBuf> {
    let canonical = fs::canonicalize(path).map_err(|error| {
        CliError::new(
            "PATH_UNAVAILABLE",
            format!("cannot resolve {role} {}: {error}", path.display()),
        )
    })?;
    if !canonical.is_dir() {
        return Err(CliError::new(
            "NOT_A_DIRECTORY",
            format!("{role} {} is not a directory", path.display()),
        ));
    }
    Ok(canonical)
}

#[cfg(test)]
#[path = "fs_private_tests.rs"]
mod private_tests;

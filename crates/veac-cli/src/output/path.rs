use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{CliError, CliResult};

pub(super) fn destination(path: &Path) -> CliResult<PathBuf> {
    let Some(file_name) = path.file_name() else {
        return Err(CliError::new(
            "INVALID_OUTPUT_PATH",
            format!("{} has no file name", path.display()),
        ));
    };
    if let Ok(metadata) = fs::symlink_metadata(path) {
        if metadata.file_type().is_symlink() {
            return Err(CliError::new(
                "OUTPUT_SYMLINK",
                format!("refusing symlink output {}", path.display()),
            ));
        }
        if metadata.is_dir() {
            return Err(CliError::new(
                "OUTPUT_IS_DIRECTORY",
                format!("output {} is a directory", path.display()),
            ));
        }
    }
    let parent = parent(path);
    let parent = fs::canonicalize(parent).map_err(|error| {
        CliError::new(
            "OUTPUT_PARENT_UNAVAILABLE",
            format!("cannot resolve output parent {}: {error}", parent.display()),
        )
    })?;
    Ok(parent.join(file_name))
}

pub(super) fn output_directory(path: &Path) -> CliResult<PathBuf> {
    let metadata = fs::symlink_metadata(path).map_err(|error| {
        CliError::new(
            "OUTPUT_DIRECTORY_REQUIRED",
            format!("multi-deliverable output must be an existing directory: {error}"),
        )
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(CliError::new(
            "OUTPUT_DIRECTORY_REQUIRED",
            "multi-deliverable output must be a non-symlink directory",
        ));
    }
    match fs::canonicalize(path) {
        Ok(path) => Ok(path),
        Err(error) => Err(CliError::new(
            "OUTPUT_DIRECTORY_REQUIRED",
            format!("cannot resolve output directory: {error}"),
        )),
    }
}

pub(super) fn normalize_input(path: &Path) -> PathBuf {
    if let Ok(canonical) = fs::canonicalize(path) {
        return canonical;
    }
    let Some(name) = path.file_name() else {
        return path.to_path_buf();
    };
    match fs::canonicalize(parent(path)) {
        Ok(parent) => parent.join(name),
        Err(_) => path.to_path_buf(),
    }
}

fn parent(path: &Path) -> &Path {
    path.parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or(Path::new("."))
}

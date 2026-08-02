use std::path::{Component, Path, PathBuf};

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

pub(super) fn checked<'a>(
    destination: &'a Path,
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<(PathBuf, &'a std::ffi::OsStr)> {
    super::checked(guard)?;
    if destination
        .components()
        .any(|part| matches!(part, Component::ParentDir))
    {
        return unsafe_path("materialization path may not contain '..'");
    }
    let name = destination.file_name().ok_or_else(|| {
        ArtifactError::new(
            ArtifactErrorKind::UnsafePath,
            "materialization destination has no file name",
        )
    })?;
    let parent = destination.parent().unwrap_or(Path::new("."));
    let parent = checked_directory(parent, guard)?;
    Ok((parent, name))
}

pub(super) fn existing(path: &Path, guard: &mut impl FnMut() -> bool) -> ArtifactResult<bool> {
    super::checked(guard)?;
    let result = std::fs::symlink_metadata(path);
    super::checked(guard)?;
    match result {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => {
            unsafe_path("materialization destination must be a regular non-symlink file")
        }
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(error.into()),
    }
}

fn checked_directory(path: &Path, guard: &mut impl FnMut() -> bool) -> ArtifactResult<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        super::step(guard, || std::env::current_dir().map_err(Into::into))?.join(path)
    };
    let metadata = super::step(guard, || {
        std::fs::symlink_metadata(&absolute).map_err(Into::into)
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return unsafe_path("materialization parent is a symlink or non-directory");
    }
    super::step(guard, || {
        std::fs::canonicalize(absolute).map_err(Into::into)
    })
}

fn unsafe_path<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(ArtifactErrorKind::UnsafePath, message))
}

#[cfg(test)]
#[path = "destination/tests.rs"]
mod tests;

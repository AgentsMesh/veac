use std::{
    fs::File,
    io::{Read, Write},
    path::{Component, Path, PathBuf},
};

use veac_artifact::{ArtifactStore, ContentDigest, OwnedStagedFile};
use veac_project::DeliveryKind;

use crate::{BuildError, BuildResult, CancellationToken};

pub(super) fn checked_root(path: impl Into<PathBuf>) -> BuildResult<PathBuf> {
    let path = path.into();
    match std::fs::create_dir_all(&path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(io_error(error)),
    }
    let metadata = std::fs::symlink_metadata(&path).map_err(io_error)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(BuildError::invalid(
            "delivery root must be a non-symlink directory",
        ));
    }
    std::fs::canonicalize(path).map_err(io_error)
}

pub(super) fn publish(
    store: &ArtifactStore,
    artifact: &ContentDigest,
    root: &Path,
    relative: &str,
    kind: DeliveryKind,
    cancellation: &CancellationToken,
) -> BuildResult<PathBuf> {
    let relative = checked_relative(relative)?;
    if kind == DeliveryKind::Directory {
        return super::directory::publish(store, artifact, root, relative, cancellation);
    }
    let parent = checked_parent(root, relative.parent().unwrap_or(Path::new("")))?;
    let Some(name) = relative.file_name() else {
        return Err(BuildError::invalid("delivery destination has no file name"));
    };
    let verified = store
        .open(artifact)
        .map_err(artifact_error)?
        .ok_or_else(|| BuildError::cache("delivery artifact is missing"))?;
    let mut source = File::open(verified.payload_path()).map_err(io_error)?;
    let mut staged = OwnedStagedFile::new_in(&parent).map_err(artifact_error)?;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        if cancellation.is_cancelled() {
            return Err(BuildError::cancelled(
                "cancelled while staging project delivery",
            ));
        }
        let read = source.read(&mut buffer).map_err(io_error)?;
        if read == 0 {
            break;
        }
        staged
            .file_mut()
            .write_all(&buffer[..read])
            .map_err(io_error)?;
    }
    staged.file_mut().flush().map_err(io_error)?;
    staged.file_mut().sync_all().map_err(io_error)?;
    let sealed = staged.seal().map_err(artifact_error)?;
    if sealed.sha256 != verified.record().content.value
        || sealed.size_bytes != verified.record().size_bytes
    {
        return Err(BuildError::cache(
            "staged delivery differs from its verified artifact",
        ));
    }
    let destination = parent.join(name);
    staged
        .persist_replace(&destination)
        .map_err(artifact_error)?;
    std::fs::canonicalize(destination).map_err(io_error)
}

fn checked_relative(value: &str) -> BuildResult<&Path> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err(BuildError::invalid(
            "delivery destination must be canonical and project-relative",
        ));
    }
    Ok(path)
}

pub(super) fn checked_parent(root: &Path, relative: &Path) -> BuildResult<PathBuf> {
    let mut current = root.to_owned();
    for part in relative.components() {
        current.push(part.as_os_str());
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_dir() => {
                return Err(BuildError::invalid(
                    "delivery parent contains a symlink or non-directory",
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                std::fs::create_dir(&current).map_err(io_error)?;
            }
            Err(error) => return Err(io_error(error)),
        }
    }
    let canonical = std::fs::canonicalize(current).map_err(io_error)?;
    if !canonical.starts_with(root) {
        return Err(BuildError::invalid("delivery parent escaped its root"));
    }
    Ok(canonical)
}

fn artifact_error(error: veac_artifact::ArtifactError) -> BuildError {
    BuildError::cache(format!("artifact delivery failure: {error}"))
}

fn io_error(error: std::io::Error) -> BuildError {
    BuildError::new(
        crate::BuildErrorKind::Cache,
        format!("delivery filesystem failure: {error}"),
    )
}

#[cfg(test)]
#[path = "delivery/tests.rs"]
mod tests;

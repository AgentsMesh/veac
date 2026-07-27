use std::path::Path;
use std::sync::OnceLock;
use std::time::Instant;

use tempfile::Builder;
use veac_artifact::{
    copy_verified_source_bounded_while, read_verified_source_bounded_while, ArtifactError,
    ArtifactErrorKind, MAX_IN_MEMORY_ARTIFACT_BYTES,
};

use super::{
    executable_source, interpreter, permissions::executable_permissions,
    permissions::master_permissions, resolve, tool_error, LaunchExecutable, PinnedExecutable,
};
use crate::RuntimeError;

impl PinnedExecutable {
    pub(crate) fn capture_until(binary: &Path, deadline: Instant) -> Result<Self, RuntimeError> {
        active(deadline)?;
        let source = resolve(binary)?;
        executable_source(&source)?;
        let directory = Builder::new()
            .prefix(".veac-tool-")
            .tempdir()
            .map_err(|error| tool_error("create executable snapshot", error))?;
        let mut path = directory.path().join("pinned");
        if let Some(extension) = source.extension() {
            path.set_extension(extension);
        }
        let copied = copy_verified_source_bounded_while(
            &source,
            &path,
            None,
            MAX_IN_MEMORY_ARTIFACT_BYTES,
            || Instant::now() < deadline,
        )
        .map_err(|error| artifact("pin executable", error))?;
        let bytes = read_verified_source_bounded_while(
            &path,
            Some(&copied.identity),
            MAX_IN_MEMORY_ARTIFACT_BYTES,
            || Instant::now() < deadline,
        )
        .map_err(|error| artifact("verify pinned executable", error))?;
        active(deadline)?;
        interpreter::validate(&bytes)?;
        master_permissions(&path, directory.path())?;
        active(deadline)?;
        Ok(Self {
            directory,
            path,
            identity: copied.identity,
        })
    }

    pub(crate) fn cached_until<'a>(
        cache: &'a OnceLock<Self>,
        binary: &Path,
        deadline: Instant,
    ) -> Result<&'a Self, RuntimeError> {
        active(deadline)?;
        if let Some(pinned) = cache.get() {
            return Ok(pinned);
        }
        let captured = match Self::capture_until(binary, deadline) {
            Ok(captured) => captured,
            Err(error) => return cache.get().ok_or(error),
        };
        let _ = cache.set(captured);
        cache
            .get()
            .ok_or_else(|| RuntimeError::new("failed to cache pinned executable"))
    }

    pub(crate) fn launch_until(&self, deadline: Instant) -> Result<LaunchExecutable, RuntimeError> {
        active(deadline)?;
        let directory = Builder::new()
            .prefix(".veac-launch-")
            .tempdir()
            .map_err(|error| tool_error("create tool launch directory", error))?;
        let mut path = directory.path().join("tool");
        if let Some(extension) = self.path.extension() {
            path.set_extension(extension);
        }
        copy_verified_source_bounded_while(
            &self.path,
            &path,
            Some(&self.identity),
            MAX_IN_MEMORY_ARTIFACT_BYTES,
            || Instant::now() < deadline,
        )
        .map_err(|error| artifact("launch pinned executable", error))?;
        executable_permissions(&path)?;
        active(deadline)?;
        Ok(LaunchExecutable {
            _directory: directory,
            path,
        })
    }
}

fn active(deadline: Instant) -> Result<(), RuntimeError> {
    if Instant::now() < deadline {
        Ok(())
    } else {
        Err(RuntimeError::resource_limit(
            "tool snapshot exceeded its wall-clock limit",
        ))
    }
}

fn artifact(operation: &str, error: ArtifactError) -> RuntimeError {
    let message = format!("cannot {operation}: {error}");
    if error.kind == ArtifactErrorKind::ResourceLimit {
        RuntimeError::resource_limit(message)
    } else {
        RuntimeError::new(message)
    }
}

#[cfg(test)]
mod tests;

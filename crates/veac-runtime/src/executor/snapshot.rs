use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;

use tempfile::{Builder, TempDir};
use veac_artifact::{
    copy_verified_source_bounded_while, ArtifactError, ArtifactErrorKind, MAX_VERIFIED_SOURCE_BYTES,
};
use veac_codegen::emitter::{BackendAction, BackendFilterBinding, BackendInput, BackendTask};

use super::contract;
use super::model::RuntimeBundle;
use crate::RuntimeError;

mod passlog;

pub(in crate::executor) use passlog::ReboundPasslogs;

#[derive(Debug)]
pub(super) struct ResourceSnapshots {
    _directory: TempDir,
    files: BTreeMap<PathBuf, PathBuf>,
    directories: BTreeMap<Vec<PathBuf>, PathBuf>,
}

pub(super) fn capture(
    bundle: &RuntimeBundle,
    deadline: Instant,
) -> Result<ResourceSnapshots, RuntimeError> {
    super::deadline::ensure_setup(deadline)?;
    let directory = match Builder::new().prefix(".veac-resources-").tempdir() {
        Ok(directory) => directory,
        Err(error) => {
            return Err(RuntimeError::new(format!(
                "cannot create resource snapshot: {error}"
            )))
        }
    };
    super::deadline::ensure_setup(deadline)?;
    let mut files = BTreeMap::new();
    for (index, resource) in bundle.protected_resources.iter().enumerate() {
        super::deadline::ensure_setup(deadline)?;
        let destination = numbered(directory.path(), "resource", index, &resource.path);
        copy_verified_source_bounded_while(
            &resource.path,
            &destination,
            Some(&resource.expected_identity),
            MAX_VERIFIED_SOURCE_BYTES,
            || Instant::now() < deadline,
        )
        .map_err(|error| snapshot_artifact(&resource.path, error))?;
        readonly(&destination)?;
        super::deadline::ensure_setup(deadline)?;
        files.insert(resource.path.clone(), destination);
    }
    let mut snapshots = ResourceSnapshots {
        _directory: directory,
        files,
        directories: BTreeMap::new(),
    };
    snapshots.capture_directories(bundle, deadline)?;
    Ok(snapshots)
}

impl ResourceSnapshots {
    pub(super) fn rebind(
        &self,
        task: &BackendTask,
        deadline: Instant,
    ) -> Result<BackendTask, RuntimeError> {
        super::deadline::ensure(deadline)?;
        let mut task = task.clone();
        let BackendAction::Ffmpeg(command) = &mut task.action else {
            return Ok(task);
        };
        command.inputs = command
            .inputs
            .iter()
            .map(|input| {
                self.file(&input.path)
                    .map(|path| BackendInput { path: path.clone() })
            })
            .collect::<Result<_, _>>()?;
        if let Some(contract) = &command.filter_contract {
            let graph = match contract.render_bound(&self.files, &self.directories) {
                Ok(graph) => graph,
                Err(error) => {
                    return Err(RuntimeError::new(format!(
                        "cannot bind filter snapshot: {error}"
                    )))
                }
            };
            contract::filter::validate_size(&graph)?;
            command.filter_graph = Some(graph);
        }
        super::deadline::ensure(deadline)?;
        Ok(task)
    }

    fn capture_directories(
        &mut self,
        bundle: &RuntimeBundle,
        deadline: Instant,
    ) -> Result<(), RuntimeError> {
        for binding in bundle
            .tasks
            .iter()
            .filter_map(|task| match &task.action {
                BackendAction::Ffmpeg(command) => command.filter_contract.as_ref(),
                BackendAction::WriteFile { .. } => None,
            })
            .flat_map(|contract| contract.bindings())
        {
            super::deadline::ensure_setup(deadline)?;
            let BackendFilterBinding::Directory { files, .. } = binding else {
                continue;
            };
            if self.directories.contains_key(files) {
                continue;
            }
            let path = self
                ._directory
                .path()
                .join(format!("fonts-{:04}", self.directories.len()));
            std::fs::create_dir(&path).map_err(snapshot_error)?;
            for (index, original) in files.iter().enumerate() {
                super::deadline::ensure_setup(deadline)?;
                let source = self.file(original)?;
                let destination = numbered(&path, "font", index, original);
                std::fs::hard_link(source, destination).map_err(snapshot_error)?;
            }
            self.directories.insert(files.clone(), path);
        }
        Ok(())
    }

    fn file(&self, path: &Path) -> Result<&PathBuf, RuntimeError> {
        self.files.get(path).ok_or_else(|| {
            RuntimeError::new(format!(
                "protected resource has no private snapshot: {}",
                path.display()
            ))
        })
    }
}

fn numbered(root: &Path, prefix: &str, index: usize, original: &Path) -> PathBuf {
    let mut path = root.join(format!("{prefix}-{index:04}"));
    if let Some(extension) = original.extension() {
        path.set_extension(extension);
    }
    path
}

fn readonly(path: &Path) -> Result<(), RuntimeError> {
    let mut permissions = std::fs::metadata(path)
        .map_err(snapshot_error)?
        .permissions();
    permissions.set_readonly(true);
    std::fs::set_permissions(path, permissions).map_err(snapshot_error)
}

fn snapshot_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!(
        "cannot materialize private resource snapshot: {error}"
    ))
}

fn snapshot_artifact(path: &Path, error: ArtifactError) -> RuntimeError {
    let message = format!(
        "cannot snapshot protected resource {}: {error}",
        path.display()
    );
    if error.kind == ArtifactErrorKind::ResourceLimit {
        RuntimeError::resource_limit(message)
    } else {
        RuntimeError::new(message)
    }
}

#[cfg(test)]
mod tests;

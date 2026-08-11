use std::path::PathBuf;

use veac_artifact::ArtifactStore;

use crate::{
    ArtifactOutputs, CancellationToken, ExecuteRequest, ExecutionError, NodeExecutor,
    ProjectAction, ProjectArtifactInput, ProjectBackend, ProjectBackendErrorKind,
    ProjectExecutionRequest,
};

mod output;

pub(super) struct ProjectNodeExecutor<B> {
    store: ArtifactStore,
    staging_root: PathBuf,
    backend: B,
}

impl<B> ProjectNodeExecutor<B> {
    pub fn new(
        store: ArtifactStore,
        staging_root: impl Into<PathBuf>,
        backend: B,
    ) -> crate::BuildResult<Self> {
        let staging_root = staging_root.into();
        std::fs::create_dir_all(&staging_root).map_err(io_error)?;
        let metadata = std::fs::symlink_metadata(&staging_root).map_err(io_error)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(crate::BuildError::invalid(
                "project staging root must be a non-symlink directory",
            ));
        }
        Ok(Self {
            store,
            staging_root: std::fs::canonicalize(staging_root).map_err(io_error)?,
            backend,
        })
    }

    pub fn artifact_store(&self) -> &ArtifactStore {
        &self.store
    }
}

impl<B: ProjectBackend> NodeExecutor<ProjectAction> for ProjectNodeExecutor<B> {
    fn implementation_identity(
        &self,
        action: &ProjectAction,
    ) -> crate::BuildResult<veac_artifact::ContentDigest> {
        self.backend
            .implementation_identity(action.kind())
            .map_err(identity_error)?
            .digest_for(action.kind())
    }

    fn execute(
        &self,
        request: ExecuteRequest<'_, ProjectAction>,
        cancellation: &CancellationToken,
    ) -> Result<ArtifactOutputs, ExecutionError> {
        if cancellation.is_cancelled() {
            return Err(ExecutionError::cancelled("project build was cancelled"));
        }
        let inputs = request
            .inputs
            .iter()
            .map(|input| {
                let artifact = self
                    .store
                    .open(&input.digest)
                    .map_err(|error| ExecutionError::failed(error.to_string()))?
                    .ok_or_else(|| {
                        ExecutionError::failed(format!(
                            "input artifact {} is missing",
                            input.digest.value
                        ))
                    })?;
                Ok(ProjectArtifactInput {
                    role: input.role.clone(),
                    producer: input.producer.clone(),
                    output: input.output.clone(),
                    artifact,
                })
            })
            .collect::<Result<Vec<_>, ExecutionError>>()?;
        let workspace = tempfile::Builder::new()
            .prefix("veac-project-node-")
            .tempdir_in(&self.staging_root)
            .map_err(|error| {
                ExecutionError::failed(format!("cannot stage project node: {error}"))
            })?;
        let produced = self
            .backend
            .execute(
                ProjectExecutionRequest {
                    node_id: request.node_id,
                    action: request.action,
                    inputs: &inputs,
                    workspace: workspace.path(),
                },
                cancellation,
            )
            .map_err(backend_error)?;
        output::publish(
            &self.store,
            request,
            workspace.path(),
            &produced,
            cancellation,
        )
    }
}

fn backend_error(error: crate::ProjectBackendError) -> ExecutionError {
    match error.kind() {
        ProjectBackendErrorKind::Failed => ExecutionError::failed(error.message()),
        ProjectBackendErrorKind::Cancelled => ExecutionError::cancelled(error.message()),
    }
}

fn identity_error(error: crate::ProjectBackendError) -> crate::BuildError {
    crate::BuildError::new(
        crate::BuildErrorKind::Internal,
        format!("project backend identity failed: {error}"),
    )
}

fn io_error(error: std::io::Error) -> crate::BuildError {
    crate::BuildError::new(
        crate::BuildErrorKind::InvalidContract,
        format!("project runtime path failure: {error}"),
    )
}

#[cfg(test)]
#[path = "executor/tests.rs"]
mod tests;

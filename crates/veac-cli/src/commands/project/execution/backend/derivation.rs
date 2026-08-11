use std::path::Path;

use veac_build::{
    CancellationToken, ProducedProjectOutput, ProjectArtifactInput, ProjectBackendError,
    ProjectComputation,
};
use veac_runtime::executor::FfmpegEnvironment;
use veac_runtime::workflow::{media_artifact_producer, MediaWorkflow};

use super::{failed, CliProjectBackend, ProjectResultExt};

mod output;
mod source;
mod spec;

pub(super) fn execute(
    backend: &CliProjectBackend,
    workspace: &Path,
    computation: &ProjectComputation,
    operation: &veac_project::MediaDerivation,
    artifacts: &[ProjectArtifactInput],
    cancellation: &CancellationToken,
) -> Result<Vec<ProducedProjectOutput>, ProjectBackendError> {
    output::check_contract(computation, operation)?;
    let artifact_spec = spec::convert(operation)?;
    let source = source::bind(
        computation,
        operation.source(),
        artifacts,
        &backend.material_root,
    )?;
    let fingerprint = FfmpegEnvironment::fingerprint(&backend.ffmpeg)
        .project_context("cannot fingerprint media derivation tool")?;
    let producer = media_artifact_producer(&fingerprint)
        .project_context("cannot identify media derivation tool")?;
    let request = veac_artifact::MediaArtifactRequest {
        source_identity: source.identity.clone(),
        producer,
        spec: artifact_spec,
    };
    let workflow =
        MediaWorkflow::from_system_tools(backend.ffmpeg.clone(), backend.ffprobe.clone());
    let derived = match workflow.derive_while(&backend.store, &source.path, &request, || {
        !cancellation.is_cancelled()
    }) {
        Ok(derived) => derived,
        Err(error) => return Err(workflow_error(error, cancellation)),
    };
    output::materialize(
        workspace,
        computation,
        &request,
        &derived.record,
        &backend.store,
        cancellation,
    )
}

fn workflow_error(
    error: veac_runtime::workflow::WorkflowError,
    cancellation: &CancellationToken,
) -> ProjectBackendError {
    if cancellation.is_cancelled() {
        ProjectBackendError::cancelled("project media derivation was cancelled")
    } else {
        failed(format!(
            "project media derivation failed ({:?}): {error}",
            error.kind
        ))
    }
}

#[cfg(test)]
#[path = "derivation/tests.rs"]
mod tests;

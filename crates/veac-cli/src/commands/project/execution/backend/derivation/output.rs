use std::path::{Path, PathBuf};

use veac_artifact::{
    copy_verified_source_bounded_while, ArtifactRecord, ArtifactStore, MediaArtifactRequest,
};
use veac_build::{
    CancellationToken, ProducedProjectOutput, ProjectBackendError, ProjectComputation,
};
use veac_ir::{HashAlgorithm, MediaIdentity};
use veac_project::{MediaDerivation, ProjectOutput};

use super::super::{failed, ProjectOptionExt, ProjectResultExt};

pub(super) fn check_contract(
    computation: &ProjectComputation,
    operation: &MediaDerivation,
) -> Result<(), ProjectBackendError> {
    let valid = matches!(
        computation.outputs.as_slice(),
        [ProjectOutput::Media { media_type, .. }]
            if *media_type == operation.output_media_type()
    );
    if valid {
        Ok(())
    } else {
        Err(failed(
            "media derivation requires exactly one compatible media output",
        ))
    }
}

pub(super) fn materialize(
    workspace: &Path,
    computation: &ProjectComputation,
    request: &MediaArtifactRequest,
    record: &ArtifactRecord,
    store: &ArtifactStore,
    cancellation: &CancellationToken,
) -> Result<Vec<ProducedProjectOutput>, ProjectBackendError> {
    let descriptor = request
        .descriptor()
        .project_context("invalid derived artifact descriptor")?;
    let artifact = store
        .open_verified_bounded(&record.key, &descriptor, record.size_bytes)
        .project_context("cannot reopen derived artifact")?
        .project_required("derived artifact disappeared from the artifact store")?;
    if artifact.record() != record {
        return Err(failed(
            "derived artifact record changed before materialization",
        ));
    }
    let output = &computation.outputs[0];
    let relative = relative_path(output, request.spec.extension());
    let destination = workspace.join(&relative);
    std::fs::create_dir_all(destination.parent().expect("output has a parent"))
        .project_context("cannot create derivation output directory")?;
    let identity = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: record.content.value.clone(),
    };
    if let Err(error) = copy_verified_source_bounded_while(
        artifact.payload_path(),
        &destination,
        Some(&identity),
        record.size_bytes,
        || !cancellation.is_cancelled(),
    ) {
        return Err(if cancellation.is_cancelled() {
            ProjectBackendError::cancelled("project media derivation was cancelled during copy")
        } else {
            failed(format!("cannot materialize derived artifact: {error}"))
        });
    }
    store
        .open_verified_bounded(&record.key, &descriptor, record.size_bytes)
        .project_context("derived artifact changed during copy")?
        .project_required("derived artifact disappeared during copy")?;
    Ok(vec![ProducedProjectOutput {
        output: output.id().clone(),
        relative_path: relative,
        semantics: veac_build::ProjectOutputSemantics::Opaque,
    }])
}

fn relative_path(output: &ProjectOutput, extension: &str) -> PathBuf {
    Path::new("outputs")
        .join(output.id().as_str())
        .join(format!("artifact.{extension}"))
}

use std::path::Path;

use veac_build::{ProjectArtifactInput, ProjectBackendError, ProjectComputation};
use veac_evidence::{BoundObservationSource, SourceBinding, SourceSpec};
use veac_ir::{HashAlgorithm, MediaIdentity};
use veac_project::ResolvedInputSource;

use super::super::{failed, inputs, CliProjectBackend};

pub(super) fn resolve(
    backend: &CliProjectBackend,
    computation: &ProjectComputation,
    artifacts: &[ProjectArtifactInput],
    sources: &[SourceSpec],
) -> Result<Vec<BoundObservationSource>, ProjectBackendError> {
    sources
        .iter()
        .map(|source| resolve_one(backend, computation, artifacts, source))
        .collect()
}

pub(super) fn verify(
    backend: &CliProjectBackend,
    computation: &ProjectComputation,
    artifacts: &[ProjectArtifactInput],
    sources: &[SourceSpec],
) -> Result<(), ProjectBackendError> {
    resolve(backend, computation, artifacts, sources).map(|_| ())
}

fn resolve_one(
    backend: &CliProjectBackend,
    computation: &ProjectComputation,
    artifacts: &[ProjectArtifactInput],
    source: &SourceSpec,
) -> Result<BoundObservationSource, ProjectBackendError> {
    let resolved = match &source.binding {
        SourceBinding::BoundInput { input_id } => {
            let Some(input) = computation
                .inputs
                .iter()
                .find(|value| value.id.as_str() == input_id)
            else {
                return Err(failed(format!("evidence input '{input_id}' is not bound")));
            };
            match &input.source {
                ResolvedInputSource::ProjectMaterial { path } => {
                    let snapshot = inputs::material::source_snapshot(computation, &input.id)?;
                    let local = backend.material_root.join(path.as_str());
                    inputs::material::verify_snapshot(&local, snapshot)?;
                    Ok(observed(local.as_path(), &snapshot.content.value))
                }
                ResolvedInputSource::Artifact { instances, output } => {
                    let value =
                        inputs::material::one_artifact(instances, output.as_str(), artifacts)?;
                    Ok(artifact(value))
                }
                _ => Err(failed(format!(
                    "evidence input '{input_id}' is not a media authority"
                ))),
            }?
        }
        SourceBinding::Artifact { .. } => {
            return Err(failed(
                "project evidence requires artifact sources through BoundInput",
            ))
        }
        SourceBinding::Deliverable { .. } => return Err(failed(
            "project evidence runs before delivery; bind the producer output as a project input",
        )),
    };
    Ok(BoundObservationSource {
        source_id: source.id.clone(),
        source: resolved,
    })
}

fn artifact(value: &ProjectArtifactInput) -> veac_runtime::observation::ObservationSource {
    observed(
        value.artifact.payload_path(),
        &value.artifact.record().content.value,
    )
}

fn observed(path: &Path, digest: &str) -> veac_runtime::observation::ObservationSource {
    veac_runtime::observation::ObservationSource {
        path: path.to_owned(),
        identity: MediaIdentity {
            algorithm: HashAlgorithm::Sha256,
            digest: digest.to_owned(),
        },
        video_stream: None,
    }
}

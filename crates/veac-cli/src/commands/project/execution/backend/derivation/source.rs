use std::path::{Path, PathBuf};

use veac_artifact::{ContentDigest, DigestAlgorithm};
use veac_build::{ProjectArtifactInput, ProjectBackendError, ProjectComputation};
use veac_project::{InputId, ResolvedInputSource};

use super::super::{failed, inputs};

pub(super) struct BoundSource {
    pub path: PathBuf,
    pub identity: ContentDigest,
}

pub(super) fn bind(
    computation: &ProjectComputation,
    source: &InputId,
    artifacts: &[ProjectArtifactInput],
    material_root: &Path,
) -> Result<BoundSource, ProjectBackendError> {
    let Some(input) = computation.inputs.iter().find(|input| input.id == *source) else {
        return Err(failed(format!(
            "media derivation source input '{source}' is missing"
        )));
    };
    match &input.source {
        ResolvedInputSource::ProjectMaterial { path } => {
            let snapshot = inputs::material::source_snapshot(computation, source)?;
            inputs::material::verify_snapshot(&material_root.join(path.as_str()), snapshot)?;
            Ok(BoundSource {
                path: material_root.join(path.as_str()),
                identity: snapshot.content.clone(),
            })
        }
        ResolvedInputSource::Artifact { instances, output } => {
            let artifact = inputs::material::one_artifact(instances, output.as_str(), artifacts)?;
            let identity = artifact.artifact.record().content.clone();
            if identity.algorithm != DigestAlgorithm::Sha256 {
                return Err(failed("media derivation source artifact requires SHA-256"));
            }
            Ok(BoundSource {
                path: artifact.artifact.payload_path().to_owned(),
                identity,
            })
        }
        _ => Err(failed(
            "media derivation source must be project material or one artifact",
        )),
    }
}

#[cfg(test)]
#[path = "source/tests.rs"]
mod tests;

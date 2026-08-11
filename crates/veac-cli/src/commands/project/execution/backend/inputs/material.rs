use std::path::Path;

use veac_artifact::{ArtifactKind, ContentDigest, DigestAlgorithm};
use veac_build::{NodeId, ProjectArtifactInput, ProjectComputation, ProjectFileSnapshot};
use veac_lang::program::{
    BuildInputDeclaration, BuildInputManifestValue, BuildInputRole, MaterialInputAuthority,
    MaterialInputKind,
};
use veac_project::{InputId, TargetInstanceId};

use super::super::failed;

pub(super) fn require_material(
    declaration: &BuildInputDeclaration,
) -> Result<(), veac_build::ProjectBackendError> {
    if declaration.role() == BuildInputRole::Material {
        Ok(())
    } else {
        Err(failed(format!(
            "project material input '{}' must use the material role",
            declaration.name()
        )))
    }
}

pub(in crate::commands::project::execution::backend) fn source_snapshot<'a>(
    computation: &'a ProjectComputation,
    input: &InputId,
) -> Result<&'a ProjectFileSnapshot, veac_build::ProjectBackendError> {
    computation
        .bound_sources
        .iter()
        .find(|value| value.input == *input)
        .map(|value| &value.snapshot)
        .ok_or_else(|| {
            failed(format!(
                "project material input '{input}' has no source snapshot"
            ))
        })
}

pub(in crate::commands::project::execution::backend) fn verify_snapshot(
    path: &Path,
    snapshot: &ProjectFileSnapshot,
) -> Result<(), veac_build::ProjectBackendError> {
    if snapshot.content.algorithm != DigestAlgorithm::Sha256 {
        return Err(failed("project source snapshot requires SHA-256"));
    }
    let identity = veac_ir::MediaIdentity {
        algorithm: veac_ir::HashAlgorithm::Sha256,
        digest: snapshot.content.value.clone(),
    };
    let verified = veac_artifact::verify_source_bounded(
        path,
        Some(&identity),
        veac_artifact::MAX_VERIFIED_SOURCE_BYTES,
    )
    .map_err(|error| {
        failed(format!(
            "project source snapshot verification failed: {error}"
        ))
    })?;
    if verified.size_bytes != snapshot.size_bytes {
        return Err(failed("project source snapshot size changed"));
    }
    Ok(())
}

pub(in crate::commands::project::execution::backend) fn one_artifact<'a>(
    instances: &[TargetInstanceId],
    output: &str,
    artifacts: &'a [ProjectArtifactInput],
) -> Result<&'a ProjectArtifactInput, veac_build::ProjectBackendError> {
    let producers = instances
        .iter()
        .map(instance_node_id)
        .collect::<Result<Vec<_>, _>>()?;
    let mut matches = artifacts
        .iter()
        .filter(|value| value.output.as_str() == output && producers.contains(&value.producer));
    let first = matches
        .next()
        .ok_or_else(|| failed("project artifact input is missing from the execution request"))?;
    if matches.next().is_some() {
        return Err(failed(
            "one material Build input cannot bind multiple project artifacts",
        ));
    }
    Ok(first)
}

fn instance_node_id(
    instance: &TargetInstanceId,
) -> Result<NodeId, veac_build::ProjectBackendError> {
    let digest = ContentDigest::sha256(instance.as_str());
    NodeId::new(format!("instance-{}", &digest.value[..32]))
        .map_err(|error| failed(error.to_string()))
}

pub(super) fn kind_from_artifact(
    kind: ArtifactKind,
) -> Result<MaterialInputKind, veac_build::ProjectBackendError> {
    match kind {
        ArtifactKind::VideoMaster => Ok(MaterialInputKind::Video),
        ArtifactKind::AudioFile => Ok(MaterialInputKind::Audio),
        ArtifactKind::StillImage => Ok(MaterialInputKind::Image),
        _ => Err(failed(format!(
            "artifact kind {kind:?} is not a VEAC material authority"
        ))),
    }
}

pub(super) fn kind_from_path(
    path: &Path,
) -> Result<MaterialInputKind, veac_build::ProjectBackendError> {
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .map(str::to_ascii_lowercase)
        .ok_or_else(|| failed("project material path requires a recognized extension"))?;
    match extension.as_str() {
        "mp4" | "mov" | "mkv" | "webm" | "m4v" => Ok(MaterialInputKind::Video),
        "wav" | "m4a" | "aac" | "flac" | "mp3" => Ok(MaterialInputKind::Audio),
        "png" | "jpg" | "jpeg" | "webp" | "tif" | "tiff" => Ok(MaterialInputKind::Image),
        "ttf" | "otf" | "ttc" => Ok(MaterialInputKind::Font),
        "lut" => Ok(MaterialInputKind::Lut1d),
        "cube" => Ok(MaterialInputKind::Lut3d),
        _ => Err(failed(format!(
            "project material extension '.{extension}' has no closed material kind"
        ))),
    }
}

pub(super) fn material_value(
    kind: MaterialInputKind,
    path: &str,
    sha256: &str,
    authority: MaterialInputAuthority,
) -> BuildInputManifestValue {
    BuildInputManifestValue::Material {
        kind,
        path: path.to_owned(),
        sha256: sha256.to_owned(),
        authority,
        video_stream: None,
        audio_stream: None,
    }
}

#[cfg(test)]
#[path = "material/tests.rs"]
mod tests;

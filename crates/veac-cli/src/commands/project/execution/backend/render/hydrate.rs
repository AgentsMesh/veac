use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;

use veac_ir::{HashAlgorithm, MaterialId, MaterialKind, MaterialSource};

use super::super::inputs::AuthorizedMaterial;
use super::super::{failed, ProjectResultExt};
use veac_lang::program::MaterialInputKind;

pub(super) fn materials(
    envelope: &mut veac_ir::ProjectEnvelope,
    required: &BTreeSet<MaterialId>,
    authorized: &BTreeMap<String, AuthorizedMaterial>,
) -> Result<BTreeMap<MaterialId, PathBuf>, veac_build::ProjectBackendError> {
    let mut paths = BTreeMap::new();
    for material in &mut envelope.project.materials {
        if !required.contains(&material.id) {
            continue;
        }
        let uri = match &material.source {
            MaterialSource::File { uri } => uri,
            MaterialSource::Remote { .. } => {
                return Err(failed("project targets cannot consume remote material"))
            }
        };
        let Some(binding) = authorized.get(uri) else {
            return Err(failed(format!(
                "material '{}' has no project input authority",
                material.id
            )));
        };
        if material_kind(binding.kind) != material.kind {
            return Err(failed(format!(
                "material '{}' kind differs from its project input",
                material.id
            )));
        }
        verify_authored_identity(material, binding)?;
        let observed = if matches!(
            material.kind,
            MaterialKind::Font | MaterialKind::Lut1d | MaterialKind::Lut3d
        ) {
            veac_runtime::asset::sha256_identity(&binding.path)
                .project_context("material identity failed")?
        } else {
            let snapshot = veac_runtime::asset::SystemFfprobe::default()
                .probe_with_intent(&binding.path, material.stream_intent.clone())
                .project_context("material probe failed")?;
            let identity = snapshot.observed_identity.clone();
            material.probe = Some(snapshot);
            identity
        };
        if observed.algorithm != HashAlgorithm::Sha256 || observed.digest != binding.sha256 {
            return Err(failed(format!(
                "material '{}' bytes differ from its project authority",
                material.id
            )));
        }
        material.identity = Some(observed);
        paths.insert(material.id.clone(), binding.path.clone());
    }
    if paths.len() != required.len() {
        return Err(failed(
            "required target material is absent from canonical IR",
        ));
    }
    Ok(paths)
}

fn verify_authored_identity(
    material: &veac_ir::Material,
    binding: &AuthorizedMaterial,
) -> Result<(), veac_build::ProjectBackendError> {
    let Some(identity) = material.identity.as_ref() else {
        return Err(failed(format!(
            "material '{}' has no authored identity",
            material.id
        )));
    };
    if identity.algorithm != HashAlgorithm::Sha256 || identity.digest != binding.sha256 {
        return Err(failed(format!(
            "material '{}' identity differs from its project input",
            material.id
        )));
    }
    Ok(())
}

fn material_kind(value: MaterialInputKind) -> MaterialKind {
    match value {
        MaterialInputKind::Video => MaterialKind::Video,
        MaterialInputKind::Audio => MaterialKind::Audio,
        MaterialInputKind::Image => MaterialKind::Image,
        MaterialInputKind::Font => MaterialKind::Font,
        MaterialInputKind::Lut1d => MaterialKind::Lut1d,
        MaterialInputKind::Lut3d => MaterialKind::Lut3d,
    }
}

#[cfg(test)]
#[path = "hydrate/tests.rs"]
mod tests;

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use veac_ir::{HashAlgorithm, MaterialId, MaterialKind, MaterialSource, MediaIdentity};

use crate::environment::Environment;
use crate::error::{CliError, CliResult};
use crate::fs;

#[derive(Debug)]
pub(crate) struct LoadedProject {
    pub envelope: veac_ir::ProjectEnvelope,
    pub project_file: PathBuf,
}

#[derive(Debug)]
pub(crate) struct HydratedProject {
    pub envelope: veac_ir::ProjectEnvelope,
    pub project_file: PathBuf,
    pub material_paths: BTreeMap<MaterialId, PathBuf>,
}

pub(crate) fn load(path: &Path) -> CliResult<veac_ir::ProjectEnvelope> {
    let input = fs::read_utf8(path, "canonical project")?;
    veac_ir::decode_canonical_json(&input).map_err(canonical_error)
}

pub(crate) fn load_local(path: &Path) -> CliResult<LoadedProject> {
    let project_file = fs::canonical_file(path, "canonical project")?;
    let envelope = load(&project_file)?;
    Ok(LoadedProject {
        envelope,
        project_file,
    })
}

pub(crate) fn hydrate(
    mut loaded: LoadedProject,
    required: &BTreeSet<MaterialId>,
    environment: &dyn Environment,
) -> CliResult<HydratedProject> {
    let project_file = loaded.project_file;
    let base = project_file.parent().unwrap_or(Path::new("."));
    let mut material_paths = BTreeMap::new();
    for material in &mut loaded.envelope.project.materials {
        if !required.contains(&material.id) {
            continue;
        }
        let uri = match &material.source {
            MaterialSource::File { uri } => uri,
            MaterialSource::Remote { uri } => {
                return Err(CliError::new(
                    "REMOTE_MATERIAL_UNRESOLVED",
                    format!("material {} is not materialized: {uri}", material.id),
                ))
            }
        };
        let local = fs::canonical_file(&base.join(uri), "material")?;
        let observed = if matches!(
            material.kind,
            MaterialKind::Font | MaterialKind::Lut1d | MaterialKind::Lut3d
        ) {
            environment.identity(&local)?
        } else {
            let snapshot = environment.probe(&local, material.stream_intent.clone())?;
            let observed = snapshot.observed_identity.clone();
            material.probe = Some(snapshot);
            observed
        };
        verify_pin(material.identity.as_ref(), &observed, &material.id)?;
        material.identity = Some(observed);
        material_paths.insert(material.id.clone(), local);
    }
    Ok(HydratedProject {
        envelope: loaded.envelope,
        project_file,
        material_paths,
    })
}

fn verify_pin(
    expected: Option<&MediaIdentity>,
    observed: &MediaIdentity,
    material_id: &MaterialId,
) -> CliResult {
    if expected.is_some_and(|value| value.algorithm != HashAlgorithm::Sha256) {
        return Err(CliError::new(
            "UNSUPPORTED_HASH_ALGORITHM",
            format!("material {material_id} uses an unsupported identity algorithm"),
        ));
    }
    if expected.is_some_and(|value| value != observed) {
        return Err(CliError::new(
            "MATERIAL_IDENTITY_MISMATCH",
            format!("material {material_id} bytes do not match its authored identity"),
        ));
    }
    Ok(())
}

fn canonical_error(error: veac_ir::CanonicalError) -> CliError {
    match error {
        veac_ir::CanonicalError::Json(error) => CliError::new("CANONICAL_JSON", error.to_string()),
        veac_ir::CanonicalError::Validation(errors) => crate::diagnostic::ir(errors.diagnostics()),
    }
}

pub(crate) fn local_material_paths(
    envelope: &veac_ir::ProjectEnvelope,
    owner_file: &Path,
) -> Vec<PathBuf> {
    let base = owner_file.parent().unwrap_or(Path::new("."));
    envelope
        .project
        .materials
        .iter()
        .filter_map(|material| match &material.source {
            MaterialSource::File { uri } => Some(base.join(uri)),
            MaterialSource::Remote { .. } => None,
        })
        .collect()
}

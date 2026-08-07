use std::collections::BTreeSet;
use std::path::Path;

use crate::environment::Environment;
use crate::error::{CliError, CliResult};

pub(crate) fn run(
    input: &Path,
    material: Option<&str>,
    material_root: Option<&Path>,
    environment: &dyn Environment,
) -> CliResult {
    let snapshot = match material {
        Some(material) => project_material(input, material, material_root, environment),
        None => media(input, environment),
    }?;
    emit(&snapshot)
}

fn media(file: &Path, environment: &dyn Environment) -> CliResult<veac_ir::MediaProbeSnapshot> {
    let file = crate::fs::canonical_file(file, "media")?;
    environment.probe(&file, veac_runtime::asset::auto_stream_intent())
}

fn project_material(
    project: &Path,
    raw_id: &str,
    material_root: Option<&Path>,
    environment: &dyn Environment,
) -> CliResult<veac_ir::MediaProbeSnapshot> {
    let material_id = veac_ir::MaterialId::new(raw_id)
        .map_err(|error| CliError::new("MATERIAL_ID_INVALID", error.to_string()))?;
    let loaded = crate::canonical::load_local(project)?;
    let material = loaded
        .envelope
        .project
        .materials
        .iter()
        .find(|material| material.id == material_id)
        .ok_or_else(|| {
            CliError::new(
                "MATERIAL_NOT_FOUND",
                format!("canonical project does not contain material {material_id}"),
            )
        })?;
    if !matches!(
        material.kind,
        veac_ir::MaterialKind::Video | veac_ir::MaterialKind::Audio | veac_ir::MaterialKind::Image
    ) {
        return Err(CliError::new(
            "MATERIAL_NOT_PROBEABLE",
            format!("material {material_id} is not audio, video, or image media"),
        ));
    }
    let required = BTreeSet::from([material_id.clone()]);
    let hydrated = crate::canonical::hydrate_with_material_root(
        loaded,
        &required,
        material_root,
        environment,
    )?;
    hydrated
        .envelope
        .project
        .materials
        .iter()
        .find(|material| material.id == material_id)
        .and_then(|material| material.probe.clone())
        .ok_or_else(|| {
            CliError::new(
                "MATERIAL_PROBE_MISSING",
                format!("material {material_id} did not produce a probe snapshot"),
            )
        })
}

fn emit(snapshot: &veac_ir::MediaProbeSnapshot) -> CliResult {
    let mut json = match serde_json::to_string_pretty(&snapshot) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("PROBE_ENCODE", error.to_string())),
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

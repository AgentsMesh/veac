use std::path::Path;

use veac_plan::ResolvedInput;

use super::{io, PackageManifest};
use crate::{
    ArtifactError, ArtifactErrorKind, ArtifactResult, ExecutionBindingManifest, ExecutionBindings,
    ExecutionInputBinding, EXECUTION_BINDING_SCHEMA_ID,
};

pub(super) fn original_path<'a>(
    bindings: &'a ExecutionBindings,
    input: &ResolvedInput,
) -> ArtifactResult<&'a Path> {
    bindings
        .input(&input.id)
        .and_then(|binding| binding.resource())
        .map(|resource| resource.path())
        .ok_or_else(|| {
            ArtifactError::new(
                ArtifactErrorKind::MissingBinding,
                format!("missing execution binding for {}", input.id),
            )
        })
}

pub fn package_binding_manifest(
    root: &Path,
    manifest: &PackageManifest,
) -> ArtifactResult<ExecutionBindingManifest> {
    manifest.validate()?;
    let inputs = manifest
        .entries
        .iter()
        .map(|entry| {
            let path = io::verified_package_file(root, &entry.packaged_path)?;
            io::verify_file(&path, &entry.identity)?;
            Ok(ExecutionInputBinding {
                input_id: entry.input_id.clone(),
                path: std::fs::canonicalize(path)?,
                identity: entry.identity.clone(),
            })
        })
        .collect::<ArtifactResult<Vec<_>>>()?;
    let bindings = ExecutionBindingManifest {
        schema: EXECUTION_BINDING_SCHEMA_ID.to_owned(),
        schema_version: 1,
        plan_hash: manifest.plan_hash.clone(),
        inputs,
    };
    bindings.validate()?;
    Ok(bindings)
}

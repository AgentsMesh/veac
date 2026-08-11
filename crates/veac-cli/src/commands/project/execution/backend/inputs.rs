use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use veac_build::{ProjectArtifactInput, ProjectComputation};
use veac_lang::program::{
    BuildInputBinding, BuildInputManifestV1, BuildInputManifestValue, ExecutableBuild,
    MaterialInputAuthority, MaterialInputKind,
};
use veac_project::{ResolvedInput, ResolvedInputSource};

use super::failed;

pub(super) mod material;
mod value;

use material::*;

pub(super) struct ProgramInputs {
    pub manifest: BuildInputManifestV1,
    pub materials: BTreeMap<String, AuthorizedMaterial>,
}

pub(super) struct AuthorizedMaterial {
    pub kind: MaterialInputKind,
    pub sha256: String,
    pub path: PathBuf,
}

pub(super) fn bind(
    program: &ExecutableBuild,
    computation: &ProjectComputation,
    artifacts: &[ProjectArtifactInput],
    material_root: &Path,
) -> Result<ProgramInputs, veac_build::ProjectBackendError> {
    let mut manifest = BuildInputManifestV1::empty();
    let mut materials = BTreeMap::new();
    for input in &computation.inputs {
        let Some(declaration) = program.build_input_declarations().get(input.id.as_str()) else {
            return Err(failed(format!(
                "project input '{}' is not declared by target source",
                input.id
            )));
        };
        let (value, material) = value(input, declaration, computation, artifacts, material_root)?;
        manifest.inputs.push(BuildInputBinding {
            name: input.id.as_str().to_owned(),
            value,
        });
        if let Some(material) = material {
            if materials.insert(material.0, material.1).is_some() {
                return Err(failed(
                    "project material URI is bound by more than one input",
                ));
            }
        }
    }
    if program.build_input_declarations().len() != manifest.inputs.len() {
        return Err(failed(
            "target source declares Build inputs absent from the project target",
        ));
    }
    Ok(ProgramInputs {
        manifest,
        materials,
    })
}

fn value(
    input: &ResolvedInput,
    declaration: &veac_lang::program::BuildInputDeclaration,
    computation: &ProjectComputation,
    artifacts: &[ProjectArtifactInput],
    material_root: &Path,
) -> Result<
    (
        BuildInputManifestValue,
        Option<(String, AuthorizedMaterial)>,
    ),
    veac_build::ProjectBackendError,
> {
    use ResolvedInputSource as Source;
    match &input.source {
        Source::Literal { value } => Ok((value::literal(value, declaration)?, None)),
        Source::ProfileBinding { profile } => {
            Ok((value::identifier(profile.as_str(), declaration)?, None))
        }
        Source::LocaleBinding { locale } => {
            Ok((value::identifier(locale.as_str(), declaration)?, None))
        }
        Source::MatrixBinding { value, .. } => {
            Ok((value::identifier(value.as_str(), declaration)?, None))
        }
        Source::ProjectMaterial { path } => {
            require_material(declaration)?;
            let snapshot = source_snapshot(computation, &input.id)?;
            let local = material_root.join(path.as_str());
            verify_snapshot(&local, snapshot)?;
            let kind = kind_from_path(&local)?;
            let value = material_value(
                kind,
                path.as_str(),
                &snapshot.content.value,
                MaterialInputAuthority::ProjectMaterial,
            );
            Ok((
                value,
                Some((
                    path.as_str().to_owned(),
                    AuthorizedMaterial {
                        kind,
                        sha256: snapshot.content.value.clone(),
                        path: local,
                    },
                )),
            ))
        }
        Source::Artifact { instances, output } => {
            require_material(declaration)?;
            let artifact = one_artifact(instances, output.as_str(), artifacts)?;
            let kind = kind_from_artifact(artifact.artifact.descriptor().kind())?;
            let key = artifact.artifact.record().key.value.clone();
            let uri = format!("artifacts/{}/{key}", input.id);
            let sha256 = artifact.artifact.record().content.value.clone();
            let value = material_value(
                kind,
                &uri,
                &sha256,
                MaterialInputAuthority::Artifact { artifact_key: key },
            );
            Ok((
                value,
                Some((
                    uri,
                    AuthorizedMaterial {
                        kind,
                        sha256,
                        path: artifact.artifact.payload_path().to_owned(),
                    },
                )),
            ))
        }
        Source::AssetFact { .. } | Source::AnalysisFact { .. } => Err(failed(
            "asset and analysis facts are not scalar Build input authorities",
        )),
    }
}

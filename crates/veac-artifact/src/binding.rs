use std::path::{Path, PathBuf};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{HashAlgorithm, MediaIdentity};
use veac_plan::{PlanInputId, ResolvedRenderPlan};

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, ExecutionBindings};

pub const EXECUTION_BINDING_SCHEMA_ID: &str = "https://veac.dev/schemas/execution-bindings";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutionBindingManifest {
    pub schema: String,
    pub schema_version: u32,
    pub plan_hash: String,
    pub inputs: Vec<ExecutionInputBinding>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExecutionInputBinding {
    pub input_id: PlanInputId,
    pub path: PathBuf,
    pub identity: MediaIdentity,
}

impl ExecutionBindingManifest {
    pub fn validate(&self) -> ArtifactResult<()> {
        if self.schema != EXECUTION_BINDING_SCHEMA_ID || self.schema_version != 1 {
            return invalid("unsupported execution-binding contract");
        }
        crate::ContentDigest {
            algorithm: crate::DigestAlgorithm::Sha256,
            value: self.plan_hash.clone(),
        }
        .validate()?;
        let mut previous: Option<&str> = None;
        for input in &self.inputs {
            if previous.is_some_and(|value| value >= input.input_id.as_str()) {
                return invalid("execution bindings must be unique and sorted");
            }
            validate_path(&input.path)?;
            validate_identity(&input.identity)?;
            previous = Some(input.input_id.as_str());
        }
        Ok(())
    }
}

pub fn binding_manifest(
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
) -> ArtifactResult<ExecutionBindingManifest> {
    let mut inputs = Vec::with_capacity(plan.inputs.len());
    for input in &plan.inputs {
        let resource = bindings
            .input(&input.id)
            .and_then(|binding| binding.resource())
            .ok_or_else(|| {
                ArtifactError::new(
                    ArtifactErrorKind::MissingBinding,
                    format!("missing execution binding for {}", input.id),
                )
            })?;
        inputs.push(ExecutionInputBinding {
            input_id: input.id.clone(),
            path: canonical_verified(resource.path(), &input.observed_identity)?,
            identity: input.observed_identity.clone(),
        });
    }
    let manifest = ExecutionBindingManifest {
        schema: EXECUTION_BINDING_SCHEMA_ID.to_owned(),
        schema_version: 1,
        plan_hash: veac_plan::plan_hash(plan).map_err(serialization)?,
        inputs,
    };
    manifest.validate()?;
    Ok(manifest)
}

pub fn resolve_binding_manifest(
    plan: &ResolvedRenderPlan,
    manifest: &ExecutionBindingManifest,
) -> ArtifactResult<ExecutionBindings> {
    manifest.validate()?;
    let expected = veac_plan::plan_hash(plan).map_err(serialization)?;
    if manifest.plan_hash != expected || manifest.inputs.len() != plan.inputs.len() {
        return invalid("execution bindings target a different render plan");
    }
    let mut paths = std::collections::BTreeMap::new();
    for (input, binding) in plan.inputs.iter().zip(&manifest.inputs) {
        if input.id != binding.input_id || input.observed_identity != binding.identity {
            return invalid("execution binding input identity does not match the render plan");
        }
        let path = canonical_verified(&binding.path, &binding.identity)?;
        paths.insert(input.id.clone(), path);
    }
    ExecutionBindings::from_originals(plan, &paths)
}

pub fn canonical_binding_bytes(value: &ExecutionBindingManifest) -> ArtifactResult<Vec<u8>> {
    value.validate()?;
    serde_json_canonicalizer::to_vec(value).map_err(serialization)
}

fn canonical_verified(path: &Path, identity: &MediaIdentity) -> ArtifactResult<PathBuf> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ArtifactError::new(
            ArtifactErrorKind::UnsafePath,
            "execution binding must be a regular non-symlink file",
        ));
    }
    let path = std::fs::canonicalize(path)?;
    crate::package::io::verify_file(&path, identity)?;
    Ok(path)
}

fn validate_path(path: &Path) -> ArtifactResult<()> {
    if path.as_os_str().is_empty() {
        invalid("execution binding path cannot be empty")
    } else {
        Ok(())
    }
}

fn validate_identity(identity: &MediaIdentity) -> ArtifactResult<()> {
    if identity.algorithm != HashAlgorithm::Sha256 {
        return invalid("execution bindings require SHA-256 input identities");
    }
    crate::ContentDigest {
        algorithm: crate::DigestAlgorithm::Sha256,
        value: identity.digest.clone(),
    }
    .validate()
}

pub(crate) fn serialization(
    error: impl std::error::Error + Send + Sync + 'static,
) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::Serialization,
        "execution bindings cannot be canonicalized",
        error,
    )
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

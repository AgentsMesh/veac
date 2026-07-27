mod binding;
pub(crate) mod io;
pub use binding::*;

use std::path::Path;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{HashAlgorithm, MaterialId, MediaIdentity, ProjectEnvelope};
use veac_plan::{plan_hash, PlanInputId, PlanSource, ResolvedRenderPlan};

use crate::{
    ArtifactError, ArtifactErrorKind, ArtifactResult, ContentDigest, ExecutionBindings,
    PACKAGE_MANIFEST_SCHEMA_ID,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackageManifest {
    pub schema: String,
    pub schema_version: u32,
    pub source: PlanSource,
    pub plan_hash: String,
    pub project_path: String,
    pub project_content: ContentDigest,
    pub entries: Vec<PackageEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackageEntry {
    pub input_id: PlanInputId,
    pub material_id: Option<MaterialId>,
    pub identity: MediaIdentity,
    pub packaged_path: String,
}

impl PackageManifest {
    pub fn validate(&self) -> ArtifactResult<()> {
        if self.schema != PACKAGE_MANIFEST_SCHEMA_ID || self.schema_version != 1 {
            return invalid(
                ArtifactErrorKind::InvalidContract,
                "unsupported package contract",
            );
        }
        let mut previous: Option<&str> = None;
        io::safe_relative_path(&self.project_path)?;
        if self.project_path == "package.json"
            || self
                .entries
                .iter()
                .any(|entry| entry.packaged_path == self.project_path)
        {
            return invalid(
                ArtifactErrorKind::InvalidContract,
                "package project path collides with package metadata or media",
            );
        }
        self.project_content.validate()?;
        ContentDigest {
            algorithm: crate::DigestAlgorithm::Sha256,
            value: self.plan_hash.clone(),
        }
        .validate()?;
        for entry in &self.entries {
            if previous.is_some_and(|value| value >= entry.input_id.as_str()) {
                return invalid(
                    ArtifactErrorKind::InvalidContract,
                    "package entries must be unique and sorted",
                );
            }
            validate_identity(&entry.identity)?;
            io::safe_relative_path(&entry.packaged_path)?;
            previous = Some(entry.input_id.as_str());
        }
        Ok(())
    }
}

pub fn package_plan(
    project: &ProjectEnvelope,
    plan: &ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    destination: &Path,
) -> ArtifactResult<PackageManifest> {
    let plan_digest = plan_hash(plan).map_err(|error| {
        ArtifactError::with_source(
            ArtifactErrorKind::Serialization,
            "render plan cannot be hashed for packaging",
            error,
        )
    })?;
    veac_ir::validate(project).map_err(|error| {
        ArtifactError::with_source(
            ArtifactErrorKind::InvalidContract,
            "package project is invalid",
            error,
        )
    })?;
    let project_bytes = veac_ir::canonical_bytes(project).map_err(|error| {
        ArtifactError::with_source(
            ArtifactErrorKind::Serialization,
            "package project cannot be canonicalized",
            error,
        )
    })?;
    let snapshot = veac_ir::snapshot_hash(project).map_err(|error| {
        ArtifactError::with_source(
            ArtifactErrorKind::Serialization,
            "package project cannot be hashed",
            error,
        )
    })?;
    if snapshot != plan.header.source.snapshot_hash {
        return invalid(
            ArtifactErrorKind::IdentityMismatch,
            "package project snapshot does not match the resolved plan",
        );
    }
    let mut entries = Vec::with_capacity(plan.inputs.len());
    for input in &plan.inputs {
        validate_identity(&input.observed_identity)?;
        let source = binding::original_path(bindings, input)?;
        io::verify_file(source, &input.observed_identity)?;
        let relative = format!("inputs/sha256/{}", input.observed_identity.digest);
        io::copy_verified(source, destination, &relative, &input.observed_identity)?;
        entries.push(PackageEntry {
            input_id: input.id.clone(),
            material_id: input.material_id.clone(),
            identity: input.observed_identity.clone(),
            packaged_path: relative,
        });
    }
    let manifest = PackageManifest {
        schema: PACKAGE_MANIFEST_SCHEMA_ID.to_owned(),
        schema_version: 1,
        source: plan.header.source.clone(),
        plan_hash: plan_digest,
        project_path: "project.veac.json".to_owned(),
        project_content: ContentDigest::sha256(&project_bytes),
        entries,
    };
    manifest.validate()?;
    io::write_project(destination, &project_bytes, &manifest.project_content)?;
    io::write_manifest(destination, &canonical_package_bytes(&manifest)?)?;
    Ok(manifest)
}

pub fn packaged_project(
    root: &Path,
    manifest: &PackageManifest,
) -> ArtifactResult<std::path::PathBuf> {
    manifest.validate()?;
    let path = io::verified_package_file(root, &manifest.project_path)?;
    io::verify_digest_file(&path, &manifest.project_content)?;
    Ok(path)
}

pub fn canonical_package_bytes(value: &PackageManifest) -> ArtifactResult<Vec<u8>> {
    value.validate()?;
    serde_json_canonicalizer::to_vec(value).map_err(|error| {
        ArtifactError::with_source(
            ArtifactErrorKind::Serialization,
            "package manifest cannot be canonicalized",
            error,
        )
    })
}

fn validate_identity(identity: &MediaIdentity) -> ArtifactResult<()> {
    if identity.algorithm != HashAlgorithm::Sha256 {
        return invalid(
            ArtifactErrorKind::UnsupportedIdentity,
            "packages currently require SHA-256 media identities",
        );
    }
    if identity.digest.len() != 64
        || !identity
            .digest
            .bytes()
            .all(|value| value.is_ascii_digit() || (b'a'..=b'f').contains(&value))
    {
        return invalid(
            ArtifactErrorKind::InvalidContract,
            "media identity must be lowercase SHA-256",
        );
    }
    Ok(())
}

fn invalid<T>(kind: ArtifactErrorKind, message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(kind, message))
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::{MaterialId, MediaIdentity};
use veac_plan::{plan_hash, PlanInputId, PlanSource, ResolvedInputKind, ResolvedRenderPlan};

use crate::{
    artifact_key, ArtifactDescriptor, ArtifactError, ArtifactErrorKind, ArtifactResult,
    ContentDigest, BUILD_MANIFEST_SCHEMA_ID,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildManifest {
    pub schema: String,
    pub schema_version: u32,
    pub source: PlanSource,
    pub plan_hash: String,
    pub output: veac_plan::ResolvedOutput,
    pub inputs: Vec<BuildInput>,
    pub tools: Vec<ToolFingerprint>,
    pub artifacts: Vec<ArtifactDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildInput {
    pub id: PlanInputId,
    pub material_id: Option<MaterialId>,
    pub kind: ResolvedInputKind,
    pub canonical_uri: String,
    pub identity: MediaIdentity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ToolFingerprint {
    pub name: String,
    pub version: String,
    pub configuration: ContentDigest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDeclaration {
    pub key: ContentDigest,
    pub descriptor: ArtifactDescriptor,
}

impl BuildManifest {
    pub fn from_plan(
        plan: &ResolvedRenderPlan,
        tools: Vec<ToolFingerprint>,
        artifacts: Vec<ArtifactDeclaration>,
    ) -> ArtifactResult<Self> {
        let value = Self {
            schema: BUILD_MANIFEST_SCHEMA_ID.to_owned(),
            schema_version: 1,
            source: plan.header.source.clone(),
            plan_hash: plan_hash(plan).map_err(|error| {
                ArtifactError::with_source(
                    ArtifactErrorKind::Serialization,
                    "render plan cannot be hashed for the build manifest",
                    error,
                )
            })?,
            output: plan.output.clone(),
            inputs: plan
                .inputs
                .iter()
                .map(|input| BuildInput {
                    id: input.id.clone(),
                    material_id: input.material_id.clone(),
                    kind: input.kind.clone(),
                    canonical_uri: input.canonical_uri.clone(),
                    identity: input.observed_identity.clone(),
                })
                .collect(),
            tools,
            artifacts,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ArtifactResult<()> {
        if self.schema != BUILD_MANIFEST_SCHEMA_ID || self.schema_version != 1 {
            return invalid("unsupported build manifest contract");
        }
        strictly_sorted(self.inputs.iter().map(|input| input.id.as_str()), "inputs")?;
        strictly_sorted(self.tools.iter().map(|tool| tool.name.as_str()), "tools")?;
        strictly_sorted(
            self.artifacts
                .iter()
                .map(|artifact| artifact.key.value.as_str()),
            "artifacts",
        )?;
        for tool in &self.tools {
            if tool.name.is_empty() || tool.version.is_empty() {
                return invalid("tool fingerprints require a name and version");
            }
            tool.configuration.validate()?;
        }
        for artifact in &self.artifacts {
            artifact.key.validate()?;
            if artifact_key(&artifact.descriptor)? != artifact.key {
                return invalid("declared artifact key does not match its descriptor");
            }
        }
        Ok(())
    }
}

pub fn canonical_manifest_bytes(value: &BuildManifest) -> ArtifactResult<Vec<u8>> {
    value.validate()?;
    serde_json_canonicalizer::to_vec(value).map_err(serialization_error)
}

pub fn build_manifest_hash(value: &BuildManifest) -> ArtifactResult<ContentDigest> {
    canonical_manifest_bytes(value).map(ContentDigest::sha256)
}

pub(crate) fn serialization_error(error: serde_json::Error) -> ArtifactError {
    ArtifactError::with_source(
        ArtifactErrorKind::Serialization,
        "build manifest cannot be canonicalized",
        error,
    )
}

fn strictly_sorted<'a>(values: impl Iterator<Item = &'a str>, label: &str) -> ArtifactResult<()> {
    let mut previous: Option<&str> = None;
    for value in values {
        if previous.is_some_and(|item| item >= value) {
            return invalid(&format!("build manifest {label} must be unique and sorted"));
        }
        previous = Some(value);
    }
    Ok(())
}

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

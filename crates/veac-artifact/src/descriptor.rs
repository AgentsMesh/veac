use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    ArtifactError, ArtifactErrorKind, ArtifactResult, ContentDigest, ARTIFACT_CONTRACT_VERSION,
    ARTIFACT_SCHEMA_ID,
};

mod dependency;
mod kind;
mod parameters;

pub use dependency::*;
pub use kind::*;
pub use parameters::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProducerFingerprint {
    pub name: String,
    pub version: String,
    pub configuration: ContentDigest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDescriptor {
    pub schema: String,
    pub schema_version: u32,
    pub producer: ProducerFingerprint,
    pub dependencies: Vec<ArtifactDependency>,
    pub parameters: ArtifactParameters,
}

impl ArtifactDescriptor {
    pub fn new(
        producer: ProducerFingerprint,
        dependencies: Vec<ArtifactDependency>,
        parameters: ArtifactParameters,
    ) -> Self {
        Self {
            schema: ARTIFACT_SCHEMA_ID.to_owned(),
            schema_version: ARTIFACT_CONTRACT_VERSION,
            producer,
            dependencies,
            parameters,
        }
    }

    pub fn kind(&self) -> ArtifactKind {
        self.parameters.kind()
    }

    pub fn validate(&self) -> ArtifactResult<()> {
        if self.schema != ARTIFACT_SCHEMA_ID || self.schema_version != ARTIFACT_CONTRACT_VERSION {
            return invalid("unsupported artifact contract");
        }
        validate_producer(&self.producer)?;
        if self.dependencies.len() > crate::MAX_ARTIFACT_DEPENDENCIES {
            return resource_limit("artifact descriptor exceeds its dependency budget");
        }
        self.parameters.validate()?;
        let mut previous = None;
        for dependency in &self.dependencies {
            dependency.validate()?;
            let current = (&dependency.role, dependency.identity.value.as_str());
            if previous.is_some_and(|value| value >= current) {
                return invalid("artifact dependencies must be unique and sorted");
            }
            previous = Some(current);
        }
        Ok(())
    }
}

fn validate_producer(value: &ProducerFingerprint) -> ArtifactResult<()> {
    if value.name.is_empty() || value.version.is_empty() {
        return invalid("artifact producer name and version must be non-empty");
    }
    if value.name.len().saturating_add(value.version.len()) > crate::MAX_ARTIFACT_JSON_STRING_BYTES
    {
        return resource_limit("artifact producer exceeds its byte budget");
    }
    value.configuration.validate()
}

pub fn canonical_descriptor_bytes(value: &ArtifactDescriptor) -> ArtifactResult<Vec<u8>> {
    value.validate()?;
    crate::json::canonical_bounded(
        value,
        crate::MAX_ARTIFACT_METADATA_BYTES,
        "artifact descriptor cannot be canonicalized",
    )
}

pub fn artifact_key(value: &ArtifactDescriptor) -> ArtifactResult<ContentDigest> {
    canonical_descriptor_bytes(value).map(ContentDigest::sha256)
}

pub(crate) fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

pub(crate) fn resource_limit<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        message,
    ))
}

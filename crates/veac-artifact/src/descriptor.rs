use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    ArtifactError, ArtifactErrorKind, ArtifactResult, ContentDigest, ARTIFACT_CONTRACT_VERSION,
    ARTIFACT_SCHEMA_ID,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactKind {
    ProxyVideo,
    ProxyAudio,
    Waveform,
    Thumbnail,
    Speech,
    Translation,
    MotionTrack,
    Matte,
    OpticalFlow,
    Analysis,
    SourceSegment,
    RenderSegment,
    CaptionSidecar,
    AudioStem,
    AudioFile,
    AnimatedImage,
    StillImage,
    AdaptivePackage,
    VideoMaster,
    ImageSequenceFrame,
    VideoWaveform,
    Vectorscope,
    Histogram,
    RenderCheckpoint,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProducerFingerprint {
    pub name: String,
    pub version: String,
    pub configuration: ContentDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDependency {
    pub role: String,
    pub identity: ContentDigest,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ArtifactDescriptor {
    pub schema: String,
    pub schema_version: u32,
    pub kind: ArtifactKind,
    pub producer: ProducerFingerprint,
    pub dependencies: Vec<ArtifactDependency>,
    pub parameters: Value,
}

impl ArtifactDescriptor {
    pub fn new(
        kind: ArtifactKind,
        producer: ProducerFingerprint,
        dependencies: Vec<ArtifactDependency>,
        parameters: Value,
    ) -> Self {
        Self {
            schema: ARTIFACT_SCHEMA_ID.to_owned(),
            schema_version: ARTIFACT_CONTRACT_VERSION,
            kind,
            producer,
            dependencies,
            parameters,
        }
    }

    pub fn validate(&self) -> ArtifactResult<()> {
        if self.schema != ARTIFACT_SCHEMA_ID || self.schema_version != ARTIFACT_CONTRACT_VERSION {
            return invalid("unsupported artifact contract");
        }
        if self.producer.name.is_empty() || self.producer.version.is_empty() {
            return invalid("artifact producer name and version must be non-empty");
        }
        if self
            .producer
            .name
            .len()
            .saturating_add(self.producer.version.len())
            > crate::MAX_ARTIFACT_JSON_STRING_BYTES
            || self.dependencies.len() > crate::MAX_ARTIFACT_DEPENDENCIES
        {
            return resource_limit("artifact descriptor exceeds its metadata shape budget");
        }
        self.producer.configuration.validate()?;
        if !self.parameters.is_object() {
            return invalid("artifact parameters must be a JSON object");
        }
        crate::json::validate_shape(&self.parameters)?;
        let mut previous: Option<(&str, &str)> = None;
        for dependency in &self.dependencies {
            dependency.identity.validate()?;
            if dependency.role.is_empty() {
                return invalid("artifact dependency role must be non-empty");
            }
            if dependency.role.len() > crate::MAX_ARTIFACT_JSON_STRING_BYTES {
                return resource_limit("artifact dependency role exceeds its byte budget");
            }
            let current = (dependency.role.as_str(), dependency.identity.value.as_str());
            if previous.is_some_and(|value| value >= current) {
                return invalid("artifact dependencies must be unique and sorted");
            }
            previous = Some(current);
        }
        Ok(())
    }
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

fn invalid<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::InvalidContract,
        message,
    ))
}

fn resource_limit<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        message,
    ))
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::RationalTime;

use crate::{
    ArtifactDependency, ArtifactDependencyRole, ArtifactDescriptor, ArtifactResult, ContentDigest,
    ProducerFingerprint,
};

mod validate;

pub const ANALYSIS_RESULT_SCHEMA_ID: &str = "https://veac.dev/schemas/analysis-result";
pub const ANALYSIS_RESULT_CONTRACT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisIngestionRequest {
    pub source_identity: ContentDigest,
    pub producer: ProducerFingerprint,
    pub result: AnalysisResultEnvelope,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisResultEnvelope {
    pub schema: String,
    pub schema_version: u32,
    pub descriptor: AnalysisDescriptor,
    pub result: AnalysisResult,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "parameters",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum AnalysisDescriptor {
    SceneBoundaries(SceneBoundaryAnalysisDescriptor),
    BeatMarkers(BeatMarkerAnalysisDescriptor),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SceneBoundaryAnalysisDescriptor {
    pub sensitivity_millionths: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BeatMarkerAnalysisDescriptor {
    pub minimum_bpm_milli: u32,
    pub maximum_bpm_milli: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum AnalysisResult {
    SceneBoundaries(SceneBoundaryAnalysisResult),
    BeatMarkers(BeatMarkerAnalysisResult),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SceneBoundaryAnalysisResult {
    pub boundaries: Vec<SceneBoundary>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BeatMarkerAnalysisResult {
    pub markers: Vec<BeatMarker>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SceneBoundary {
    pub at: RationalTime,
    pub confidence_millionths: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BeatMarker {
    pub at: RationalTime,
    pub confidence_millionths: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AnalysisArtifactParameters {
    pub descriptor: AnalysisDescriptor,
    pub result_digest: ContentDigest,
}

impl AnalysisResultEnvelope {
    pub fn new(descriptor: AnalysisDescriptor, result: AnalysisResult) -> ArtifactResult<Self> {
        let value = Self {
            schema: ANALYSIS_RESULT_SCHEMA_ID.to_owned(),
            schema_version: ANALYSIS_RESULT_CONTRACT_VERSION,
            descriptor,
            result,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ArtifactResult<()> {
        validate::envelope(self)
    }

    pub fn canonical_bytes(&self, max_bytes: u64) -> ArtifactResult<Vec<u8>> {
        self.validate()?;
        crate::json::canonical_bounded(self, max_bytes, "analysis result cannot be canonicalized")
    }
}

impl AnalysisIngestionRequest {
    pub fn descriptor(&self) -> ArtifactResult<ArtifactDescriptor> {
        self.validate()?;
        let payload = self
            .result
            .canonical_bytes(crate::MAX_ANALYSIS_PAYLOAD_BYTES)?;
        let parameters = AnalysisArtifactParameters {
            descriptor: self.result.descriptor.clone(),
            result_digest: ContentDigest::sha256(payload),
        };
        let value = ArtifactDescriptor::new(
            self.producer.clone(),
            vec![ArtifactDependency::new(
                ArtifactDependencyRole::Input,
                self.source_identity.clone(),
            )],
            crate::ArtifactParameters::Analysis(parameters),
        );
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> ArtifactResult<()> {
        self.source_identity.validate()?;
        self.result.validate()?;
        validate::producer(&self.producer)
    }
}

impl AnalysisArtifactParameters {
    pub(crate) fn validate(&self) -> ArtifactResult<()> {
        validate::descriptor(&self.descriptor)?;
        self.result_digest.validate()
    }
}

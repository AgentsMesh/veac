use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::MediaIdentity;

use crate::{canonical_json, sha256_hex, BoundObservationSource, BundleError, ObservationPlanV1};

pub const EVIDENCE_PROVENANCE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceProvenanceV1 {
    pub schema_version: u32,
    pub suite_sha256: String,
    pub observation_plan_sha256: String,
    pub sources: Vec<EvidenceSourceProvenance>,
    pub producer: EvidenceProducerFingerprint,
    pub project: Option<ProjectEvidenceContext>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSourceProvenance {
    pub source_id: String,
    pub identity: MediaIdentity,
    pub video_stream: Option<u32>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceProducerFingerprint {
    pub engine: String,
    pub version: String,
    pub configuration_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectEvidenceContext {
    pub manifest_sha256: String,
    pub target_instance_id: String,
}

pub fn build_provenance(
    plan: &ObservationPlanV1,
    bindings: &[BoundObservationSource],
    producer: EvidenceProducerFingerprint,
    project: Option<ProjectEvidenceContext>,
) -> Result<EvidenceProvenanceV1, BundleError> {
    let mut sources = bindings
        .iter()
        .map(|binding| EvidenceSourceProvenance {
            source_id: binding.source_id.clone(),
            identity: binding.source.identity.clone(),
            video_stream: binding.source.video_stream,
        })
        .collect::<Vec<_>>();
    sources.sort_by(|left, right| left.source_id.cmp(&right.source_id));
    Ok(EvidenceProvenanceV1 {
        schema_version: EVIDENCE_PROVENANCE_SCHEMA_VERSION,
        suite_sha256: plan.suite_sha256.clone(),
        observation_plan_sha256: sha256_hex(&canonical_json(plan)?),
        sources,
        producer,
        project,
    })
}

pub fn evidence_cache_key(provenance: &EvidenceProvenanceV1) -> Result<String, BundleError> {
    Ok(sha256_hex(&canonical_json(provenance)?))
}

impl From<veac_runtime::executor::FfmpegFingerprint> for EvidenceProducerFingerprint {
    fn from(value: veac_runtime::executor::FfmpegFingerprint) -> Self {
        Self {
            engine: "ffmpeg".into(),
            version: value.version,
            configuration_sha256: value.configuration.value,
        }
    }
}

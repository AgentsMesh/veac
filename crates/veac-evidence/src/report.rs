use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::AssertionKind;

pub const EVIDENCE_REPORT_SCHEMA_VERSION: u32 = 1;
pub const EVIDENCE_BUNDLE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOutcome {
    Pass,
    Fail,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssertionStatus {
    Pass,
    Fail,
    Error,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AssertionResult {
    pub id: String,
    pub kind: AssertionKind,
    pub status: AssertionStatus,
    pub message: String,
    pub metrics: BTreeMap<String, f64>,
    pub series: BTreeMap<String, Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceReportV1 {
    pub schema_version: u32,
    pub suite_id: String,
    pub outcome: EvidenceOutcome,
    pub assertions: Vec<AssertionResult>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BundleArtifactRecord {
    pub path: String,
    pub size_bytes: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceBundleManifestV1 {
    pub schema_version: u32,
    pub suite_sha256: String,
    pub provenance_sha256: String,
    pub report_sha256: String,
    pub cache_key: String,
    pub outcome: EvidenceOutcome,
    pub artifacts: Vec<BundleArtifactRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundleArtifactContent {
    pub path: String,
    pub bytes: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedEvidenceBundle {
    pub manifest: EvidenceBundleManifestV1,
    pub artifacts: Vec<BundleArtifactContent>,
}

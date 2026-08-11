use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::AssertionSpec;
use crate::RationalTime;

pub const EVIDENCE_SUITE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EvidenceSuiteV1 {
    pub schema_version: u32,
    pub id: String,
    pub sources: Vec<SourceSpec>,
    pub samples: Vec<SampleSpec>,
    pub regions: Vec<RegionSpec>,
    pub assertions: Vec<AssertionSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceSpec {
    pub id: String,
    pub binding: SourceBinding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceBinding {
    Deliverable { deliverable_id: String },
    Artifact { artifact_id: String },
    BoundInput { input_id: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SampleSpec {
    pub id: String,
    pub source_id: String,
    pub at: RationalTime,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RegionSpec {
    pub id: String,
    pub space: RegionSpace,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum RegionSpace {
    Normalized {
        x: f64,
        y: f64,
        width: f64,
        height: f64,
    },
    Pixels {
        x: u32,
        y: u32,
        width: u32,
        height: u32,
    },
}

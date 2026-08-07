use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{CaptionCue, CaptionDocumentNative, CaptionStyle, CURRENT_SCHEMA_VERSION, SCHEMA_ID};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionEnvelope {
    pub schema: String,
    pub schema_version: u32,
    pub document: CaptionDocument,
}

impl CaptionEnvelope {
    pub fn new(document: CaptionDocument) -> Self {
        Self {
            schema: SCHEMA_ID.to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
            document,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CaptionDocument {
    pub timescale: u32,
    pub language: Option<String>,
    pub overlap_policy: OverlapPolicy,
    pub native: Option<CaptionDocumentNative>,
    pub styles: Vec<CaptionStyle>,
    pub cues: Vec<CaptionCue>,
}

impl CaptionDocument {
    pub fn new(timescale: u32, overlap_policy: OverlapPolicy) -> Self {
        Self {
            timescale,
            language: None,
            overlap_policy,
            native: None,
            styles: Vec::new(),
            cues: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum OverlapPolicy {
    Reject,
    Allow,
}

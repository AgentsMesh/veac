use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{TemporalDefinitionId, TemporalLogicalKey, TemporalProvenanceId, TemporalSourceId};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalSourceSpan {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TemporalDefinitionKind {
    Function,
    Closure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalAuthoredSite {
    pub definition_id: TemporalDefinitionId,
    pub function: String,
    pub source_id: TemporalSourceId,
    pub span: TemporalSourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalDefinitionSite {
    pub id: TemporalDefinitionId,
    pub kind: TemporalDefinitionKind,
    pub name: String,
    pub source_id: TemporalSourceId,
    pub span: TemporalSourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalProvenance {
    pub id: TemporalProvenanceId,
    pub definition: TemporalDefinitionSite,
    pub origin: TemporalAuthoredSite,
    pub call_stack: Vec<TemporalAuthoredSite>,
    pub logical_keys: Vec<TemporalLogicalKey>,
}

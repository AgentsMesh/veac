use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    TemporalBinding, TemporalInputDeclaration, TemporalNode, TemporalNodeId, TemporalProgramId,
    TemporalProvenance, TemporalProvenanceId, TemporalType,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalProgram {
    pub id: TemporalProgramId,
    pub opset_version: u16,
    pub inputs: Vec<TemporalInputDeclaration>,
    pub result_type: TemporalType,
    pub nodes: Vec<TemporalNode>,
    pub result: TemporalNodeId,
    pub content_sha256: String,
    pub provenance_id: TemporalProvenanceId,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TemporalProgramLibrary {
    pub opset_version: u16,
    pub programs: Vec<TemporalProgram>,
    pub bindings: Vec<TemporalBinding>,
    pub provenance: Vec<TemporalProvenance>,
}

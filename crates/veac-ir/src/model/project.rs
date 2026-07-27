use std::collections::BTreeMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    Annotation, Material, MulticamGroup, OperationId, ProjectId, Relation, RenderConfig, Sequence,
    SequenceId, CURRENT_SCHEMA_VERSION, MIN_READER_VERSION, SCHEMA_ID,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectEnvelope {
    pub schema: String,
    pub schema_version: u32,
    pub min_reader_version: u32,
    pub project: Project,
}

impl ProjectEnvelope {
    pub fn new(project: Project) -> Self {
        Self {
            schema: SCHEMA_ID.to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
            min_reader_version: MIN_READER_VERSION,
            project,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub id: ProjectId,
    #[schemars(range(max = 9007199254740991u64))]
    pub revision: u64,
    pub timebase: u32,
    pub entry_sequence_id: SequenceId,
    pub render_configs: Vec<RenderConfig>,
    pub materials: Vec<Material>,
    pub multicam_groups: Vec<MulticamGroup>,
    pub annotations: Vec<Annotation>,
    pub relations: Vec<Relation>,
    pub sequences: Vec<Sequence>,
    pub applied_operations: Vec<AppliedOperation>,
    pub metadata: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AppliedOperation {
    pub id: OperationId,
    /// SHA-256 of the RFC 8785 canonical EditBatch. Reusing an ID for different content conflicts.
    pub request_hash: String,
}

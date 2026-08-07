use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    Annotation, ExecutableManifest, Material, MulticamGroup, OperationId, ProjectAuthorship,
    ProjectId, Relation, RenderConfig, Sequence, SequenceId, TemporalProgramLibrary,
    CURRENT_SCHEMA_VERSION, MIN_READER_VERSION, SCHEMA_ID,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectEnvelope {
    pub schema: String,
    pub schema_version: u32,
    pub min_reader_version: u32,
    pub executable: ExecutableManifest,
    pub temporal: TemporalProgramLibrary,
    pub project: Project,
}

impl ProjectEnvelope {
    pub fn new(
        project: Project,
        executable: ExecutableManifest,
        temporal: TemporalProgramLibrary,
    ) -> Self {
        Self {
            schema: SCHEMA_ID.to_owned(),
            schema_version: CURRENT_SCHEMA_VERSION,
            min_reader_version: MIN_READER_VERSION,
            executable,
            temporal,
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
    pub authorship: Option<ProjectAuthorship>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AppliedOperation {
    pub id: OperationId,
    /// SHA-256 of the RFC 8785 canonical EditBatch. Reusing an ID for different content conflicts.
    pub request_hash: String,
}

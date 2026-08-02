use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::OperationId;

use super::{ExpressionSite, SourceNodeRef};

pub const SOURCE_EDIT_SCHEMA: &str = "https://veac.dev/schemas/source-edit";
pub const SOURCE_EDIT_SCHEMA_VERSION: u32 = 1;
pub const MAX_SOURCE_EDIT_OPERATIONS: usize = 4_096;
pub const MAX_SOURCE_EDIT_PRECONDITIONS: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRevision {
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub source_graph_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceEditBatch {
    pub schema: String,
    #[schemars(range(min = 1, max = 1))]
    pub schema_version: u32,
    pub operation_id: OperationId,
    pub base_revision: SourceRevision,
    pub atomic: bool,
    #[schemars(length(max = 4096))]
    pub preconditions: Vec<SourcePrecondition>,
    #[schemars(length(min = 1, max = 4096))]
    pub operations: Vec<SourceEditOperation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourcePrecondition {
    NodeExists {
        target: SourceNodeRef,
    },
    NodeAbsent {
        target: SourceNodeRef,
    },
    ExpressionEquals {
        target: SourceNodeRef,
        site: ExpressionSite,
        expression: ExpressionSource,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceEditOperation {
    SetExpression {
        target: SourceNodeRef,
        site: ExpressionSite,
        expression: ExpressionSource,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpressionSource {
    #[schemars(length(min = 1, max = 65536))]
    pub source: String,
}

impl SourceEditBatch {
    pub fn new(operation_id: OperationId, revision: SourceRevision) -> Self {
        Self {
            schema: SOURCE_EDIT_SCHEMA.to_owned(),
            schema_version: SOURCE_EDIT_SCHEMA_VERSION,
            operation_id,
            base_revision: revision,
            atomic: true,
            preconditions: Vec::new(),
            operations: Vec::new(),
        }
    }
}

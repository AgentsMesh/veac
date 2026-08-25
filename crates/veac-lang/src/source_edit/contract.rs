use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use veac_ir::OperationId;

use super::{
    BodySite, DeclarationSite, ExpressionSite, ImportSource, SourceImportRef, SourceModuleAnchor,
    SourceNodeRef, StatementSite, TopLevelDeclarationSource,
};

pub const SOURCE_EDIT_SCHEMA: &str = "https://veac.dev/schemas/source-edit";
pub const SOURCE_EDIT_SCHEMA_VERSION: u32 = 9;
pub const MAX_SOURCE_EDIT_OPERATIONS: usize = 4_096;
pub const MAX_SOURCE_EDIT_PRECONDITIONS: usize = 4_096;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredSourceRevision {
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub source_graph_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceRevision {
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub authored_source_graph_sha256: String,
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub complete_source_graph_sha256: String,
}

impl SourceRevision {
    pub fn new(authored: &AuthoredSourceRevision, complete_source_graph_sha256: &str) -> Self {
        Self {
            authored_source_graph_sha256: authored.source_graph_sha256.clone(),
            complete_source_graph_sha256: complete_source_graph_sha256.to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceEditBatch {
    #[schemars(extend("const" = SOURCE_EDIT_SCHEMA))]
    pub schema: String,
    #[schemars(extend("const" = SOURCE_EDIT_SCHEMA_VERSION))]
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
    StatementEquals {
        target: SourceNodeRef,
        site: StatementSite,
        statement: StatementSource,
    },
    BodyEquals {
        target: SourceNodeRef,
        site: BodySite,
        body: BodySource,
    },
    DeclarationEquals {
        target: SourceNodeRef,
        site: DeclarationSite,
        declaration: DeclarationSource,
    },
    TopLevelDeclarationEquals {
        target: SourceNodeRef,
        declaration: TopLevelDeclarationSource,
    },
    ImportExists {
        target: SourceImportRef,
    },
    ImportAbsent {
        target: SourceImportRef,
    },
    ImportEquals {
        target: SourceImportRef,
        import: ImportSource,
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
    SetStatement {
        target: SourceNodeRef,
        site: StatementSite,
        statement: StatementSource,
    },
    SetBody {
        target: SourceNodeRef,
        site: BodySite,
        body: BodySource,
    },
    SetDeclaration {
        target: SourceNodeRef,
        site: DeclarationSite,
        declaration: DeclarationSource,
    },
    SetTopLevelDeclaration {
        target: SourceNodeRef,
        declaration: TopLevelDeclarationSource,
    },
    InsertDeclaration {
        module: String,
        anchor: SourceModuleAnchor,
        declaration: TopLevelDeclarationSource,
    },
    RemoveDeclaration {
        target: SourceNodeRef,
    },
    InsertImport {
        module: String,
        anchor: SourceModuleAnchor,
        import: ImportSource,
    },
    RemoveImport {
        target: SourceImportRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExpressionSource {
    #[schemars(length(min = 1, max = 65536))]
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StatementSource {
    #[schemars(length(min = 1, max = 65536))]
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BodySource {
    #[schemars(length(min = 1, max = 65536))]
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DeclarationSource {
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

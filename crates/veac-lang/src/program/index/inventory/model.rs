use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::program::BuildInputRole;
use crate::source_edit::{
    BodySite, DeclarationSite, ExpressionSite, SourceImportRef, SourceNodeRef, SourceRevision,
    StatementSite, TextRange,
};

use super::{SOURCE_INDEX_SCHEMA, SOURCE_INDEX_SCHEMA_VERSION};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexInventory {
    #[schemars(extend("const" = SOURCE_INDEX_SCHEMA))]
    pub schema: String,
    #[schemars(extend("const" = SOURCE_INDEX_SCHEMA_VERSION))]
    pub schema_version: u32,
    pub revision: SourceRevision,
    pub build_inputs: Vec<SourceIndexBuildInput>,
    pub modules: Vec<SourceIndexModule>,
    pub nodes: Vec<SourceIndexNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexBuildInput {
    pub name: String,
    pub role: BuildInputRole,
    pub value_type: SourceIndexBuildInputType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceIndexBuildInputType {
    Bool,
    #[serde(rename = "int")]
    Integer,
    Scalar,
    Text,
    Time,
    Length,
    Angle,
    Color,
    Enum {
        name: String,
        type_id: String,
        definition_sha256: String,
        variants: Vec<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexModule {
    pub module: String,
    pub range: TextRange,
    pub imports: Vec<SourceIndexImport>,
    pub declarations: Vec<SourceIndexTopLevelDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexImport {
    pub target: SourceImportRef,
    pub path: String,
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexTopLevelDeclaration {
    pub target: SourceNodeRef,
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexNode {
    pub target: SourceNodeRef,
    pub range: TextRange,
    pub expressions: Vec<SourceIndexExpression>,
    pub statements: Vec<SourceIndexStatement>,
    pub bodies: Vec<SourceIndexBody>,
    pub declarations: Vec<SourceIndexDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexExpression {
    pub site: ExpressionSite,
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexStatement {
    pub site: StatementSite,
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexBody {
    pub site: BodySite,
    pub source: String,
    pub range: TextRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SourceIndexDeclaration {
    pub site: DeclarationSite,
    pub source: String,
    pub range: TextRange,
}

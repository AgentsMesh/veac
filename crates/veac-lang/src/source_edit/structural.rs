use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::SourceNodeRef;

#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(deny_unknown_fields)]
pub struct SourceImportRef {
    pub module: String,
    pub alias: String,
}

impl SourceImportRef {
    pub fn new(module: impl Into<String>, alias: impl Into<String>) -> Self {
        Self {
            module: module.into(),
            alias: alias.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum SourceModuleAnchor {
    ModuleStart,
    ModuleEnd,
    BeforeDeclaration { target: SourceNodeRef },
    AfterDeclaration { target: SourceNodeRef },
    BeforeImport { target: SourceImportRef },
    AfterImport { target: SourceImportRef },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TopLevelDeclarationSource {
    #[schemars(length(min = 1, max = 65536))]
    pub source: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ImportSource {
    #[schemars(length(min = 1, max = 4096))]
    pub path: String,
    #[schemars(length(min = 1, max = 128))]
    pub alias: String,
}

impl ImportSource {
    pub(crate) fn render(&self) -> String {
        format!(
            "import {} as {};",
            crate::string_codec::quote(&self.path),
            self.alias
        )
    }
}

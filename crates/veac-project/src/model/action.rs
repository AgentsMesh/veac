use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{MediaDerivation, ProjectPath};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectTargetEntry {
    Veac { source: ProjectPath },
    MediaDerivation { operation: MediaDerivation },
    Evidence { contract: ProjectPath },
}

impl ProjectTargetEntry {
    pub fn source_path(&self) -> Option<&ProjectPath> {
        match self {
            Self::Veac { source } => Some(source),
            Self::Evidence { contract } => Some(contract),
            Self::MediaDerivation { .. } => None,
        }
    }
}

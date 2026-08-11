use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{AxisId, FactId, InputId, OutputId, ProjectLiteral, ProjectPath, TargetRef};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectInput {
    pub id: InputId,
    pub source: ProjectInputSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ProjectInputSource {
    Literal {
        value: ProjectLiteral,
    },
    ProfileBinding {},
    LocaleBinding {},
    MatrixBinding {
        axis: AxisId,
    },
    ProjectMaterial {
        path: ProjectPath,
    },
    Artifact {
        target: TargetRef,
        output: OutputId,
    },
    AssetFact {
        path: ProjectPath,
        fact: FactId,
    },
    AnalysisFact {
        target: TargetRef,
        output: OutputId,
        fact: FactId,
    },
}

impl ProjectInputSource {
    pub fn target_ref(&self) -> Option<(&TargetRef, &OutputId)> {
        match self {
            Self::Artifact { target, output } | Self::AnalysisFact { target, output, .. } => {
                Some((target, output))
            }
            Self::Literal { .. }
            | Self::ProfileBinding {}
            | Self::LocaleBinding {}
            | Self::MatrixBinding { .. }
            | Self::ProjectMaterial { .. }
            | Self::AssetFact { .. } => None,
        }
    }
}

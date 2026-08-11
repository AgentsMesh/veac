use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    AxisId, AxisValue, DeliveryId, FactId, InputId, MatrixAssignment, OutputId, ProfileId,
    ProjectLiteral, ProjectOutput, ProjectPath, ProjectTargetEntry, TargetId, TargetInstanceId,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTargetGraph {
    pub version: u32,
    pub manifest_digest: String,
    pub instances: Vec<TargetInstance>,
    pub edges: Vec<ResolvedTargetEdge>,
    pub build_order: Vec<TargetInstanceId>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TargetInstance {
    pub id: TargetInstanceId,
    pub target: TargetId,
    pub profile: Option<ProfileId>,
    pub locale: Option<super::LocaleId>,
    pub matrix: MatrixAssignment,
    pub entry: ProjectTargetEntry,
    pub inputs: Vec<ResolvedInput>,
    pub outputs: Vec<ProjectOutput>,
    pub deliveries: Vec<ResolvedDelivery>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedInput {
    pub id: InputId,
    pub source: ResolvedInputSource,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum ResolvedInputSource {
    Literal {
        value: ProjectLiteral,
    },
    ProfileBinding {
        profile: ProfileId,
    },
    LocaleBinding {
        locale: super::LocaleId,
    },
    MatrixBinding {
        axis: AxisId,
        value: AxisValue,
    },
    ProjectMaterial {
        path: ProjectPath,
    },
    Artifact {
        instances: Vec<TargetInstanceId>,
        output: OutputId,
    },
    AssetFact {
        path: ProjectPath,
        fact: FactId,
    },
    AnalysisFact {
        instances: Vec<TargetInstanceId>,
        output: OutputId,
        fact: FactId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedDelivery {
    pub id: DeliveryId,
    pub output: OutputId,
    pub kind: DeliveryKind,
    pub destination: ProjectPath,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DeliveryKind {
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ResolvedTargetEdge {
    pub dependency: TargetInstanceId,
    pub consumer: TargetInstanceId,
    pub binding: Option<InputId>,
    pub output: Option<OutputId>,
}

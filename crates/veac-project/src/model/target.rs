use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    AxisId, AxisValue, ProfileId, ProjectDelivery, ProjectInput, ProjectOutput, ProjectTargetEntry,
    TargetId, TargetRef,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ProjectTarget {
    pub id: TargetId,
    pub entry: ProjectTargetEntry,
    pub localized: bool,
    pub inputs: Vec<ProjectInput>,
    pub profiles: Vec<ProfileId>,
    pub axes: Vec<MatrixAxis>,
    pub needs: Vec<TargetRef>,
    pub outputs: Vec<ProjectOutput>,
    pub deliveries: Vec<ProjectDelivery>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatrixAxis {
    pub id: AxisId,
    pub values: Vec<AxisValue>,
}

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiCallableSemantics {
    pub effect: ApiSemanticEffect,
    pub contains_local_mutation: bool,
    pub result: ApiResultSemantics,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiResultSemantics {
    pub shape: ApiStage,
    pub leaf: ApiStage,
    pub receiver: Option<ApiParameterDependency>,
    pub parameters: Vec<ApiParameterDependency>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ApiParameterDependency {
    pub shape_from_shape: bool,
    pub shape_from_leaf: bool,
    pub leaf_from_shape: bool,
    pub leaf_from_leaf: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiSemanticEffect {
    Pure,
    LocalMutation,
    GraphEmit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiStage {
    Const,
    Build,
    Temporal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApiEffect {
    Pure,
    Local,
    Emit,
    Any,
}

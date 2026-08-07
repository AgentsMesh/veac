use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{AuthoredDefinitionId, AuthoredFunctionName, AuthoredSourceId, LoopLogicalKey};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthorshipEventKind {
    Construct,
    Constructor,
    Update,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthoredDefinitionKind {
    Function,
    Closure,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct DomainOpcode(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredSpan {
    #[schemars(range(max = 9007199254740991u64))]
    pub start: u64,
    #[schemars(range(max = 9007199254740991u64))]
    pub end: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredSite {
    pub definition: AuthoredDefinitionId,
    pub function: AuthoredFunctionName,
    pub source: AuthoredSourceId,
    pub span: AuthoredSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredDefinition {
    pub identity: AuthoredDefinitionId,
    pub kind: AuthoredDefinitionKind,
    pub name: AuthoredFunctionName,
    pub source: AuthoredSourceId,
    pub span: AuthoredSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthoredIteration {
    pub definition: AuthoredDefinitionId,
    pub function: AuthoredFunctionName,
    pub source: AuthoredSourceId,
    pub loop_span: AuthoredSpan,
    pub binding_span: AuthoredSpan,
    #[schemars(range(max = 9007199254740991u64))]
    pub index: u64,
    pub logical_key: LoopLogicalKey,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AuthorshipEvent {
    pub kind: AuthorshipEventKind,
    pub operation: DomainOpcode,
    pub origin: AuthoredSite,
    pub definition: AuthoredDefinition,
    pub call_stack: Vec<AuthoredSite>,
    pub iterations: Vec<AuthoredIteration>,
}

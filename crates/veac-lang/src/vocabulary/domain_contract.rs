use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub(super) mod catalog;
mod identity;
mod validation;

#[cfg(test)]
#[path = "domain_contract/identity_tests.rs"]
mod identity_tests;
#[cfg(test)]
#[path = "domain_contract/temporal_tests.rs"]
mod temporal_tests;

use super::VocabularyValidationError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DomainOpsetSpec {
    pub version: u16,
    #[schemars(regex(pattern = r"^[0-9a-f]{64}$"))]
    pub registry_digest: String,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub types: Vec<DomainTypeSpec>,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub operations: Vec<DomainOperationSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DomainTypeSpec {
    pub opcode: u16,
    #[schemars(length(min = 1))]
    pub name: String,
    pub container: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DomainOperationSpec {
    pub opcode: u16,
    #[schemars(length(min = 1))]
    pub name: String,
    pub contract: DomainOperationSignature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DomainOperationSignature {
    #[schemars(extend("uniqueItems" = true))]
    pub ordered_operands: Vec<DomainOperandSpec>,
    pub result: DomainValueShapeSpec,
    pub instruction: DomainInstructionSpec,
    pub runtime_action: DomainRuntimeActionSpec,
    pub effect: DomainEffectSpec,
    pub max_stage: DomainMaxStageSpec,
    pub temporal_lowering: Option<TemporalLoweringOpcodeSpec>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DomainOperandSpec {
    #[schemars(length(min = 1))]
    pub name: String,
    pub shape: DomainValueShapeSpec,
    pub axis: DomainOperandAxisSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum DomainValueShapeSpec {
    Primitive {
        #[schemars(length(min = 1))]
        name: String,
    },
    PrimitiveList {
        #[schemars(length(min = 1))]
        name: String,
    },
    Domain {
        type_opcode: u16,
        #[schemars(length(min = 1))]
        type_name: String,
    },
    DomainList {
        type_opcode: u16,
        #[schemars(length(min = 1))]
        type_name: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DomainOperandAxisSpec {
    Topology,
    Leaf,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DomainInstructionSpec {
    DomainConstruct,
    GraphEmit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DomainRuntimeActionSpec {
    Description,
    EntityConstructor,
    OwnedAttachment,
    NonOwningUpdate,
    ProjectEntry,
    RelationConstructor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DomainEffectSpec {
    Pure,
    LocalMutation,
    GraphEmit,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DomainMaxStageSpec {
    Build,
    Temporal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TemporalLoweringOpcodeSpec {
    #[serde(rename = "compose_vector")]
    Vector,
    #[serde(rename = "compose_point")]
    Point,
    #[serde(rename = "compose_rect")]
    Rect,
}

impl DomainOpsetSpec {
    pub(crate) fn current() -> Self {
        catalog::current()
    }

    pub fn validate(&self) -> Result<(), VocabularyValidationError> {
        validation::validate(self)
    }

    pub(crate) fn validate_with_plugins(
        &self,
        plugins: &[super::PluginEffectSpec],
    ) -> Result<(), VocabularyValidationError> {
        validation::validate_with_plugins(self, plugins)
    }
}

impl Default for DomainOpsetSpec {
    fn default() -> Self {
        Self::current()
    }
}

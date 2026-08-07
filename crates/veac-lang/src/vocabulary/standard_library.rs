use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{DomainOperationSignature, VocabularyValidationError};

mod catalog;
mod validation;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StandardLibrarySpec {
    pub domain_opset_version: u16,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub types: Vec<StandardLibraryType>,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub free_functions: Vec<StandardLibraryFunction>,
    #[schemars(length(min = 1), extend("uniqueItems" = true))]
    pub methods: Vec<StandardLibraryMethod>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StandardLibraryType {
    #[schemars(length(min = 1))]
    pub name: String,
    pub domain_type_opcode: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StandardLibraryFunction {
    #[schemars(length(min = 1))]
    pub name: String,
    pub operation_opcode: u16,
    pub contract: DomainOperationSignature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct StandardLibraryMethod {
    pub receiver: DomainTypeReferenceSpec,
    #[schemars(length(min = 1))]
    pub name: String,
    pub operation_opcode: u16,
    pub contract: DomainOperationSignature,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct DomainTypeReferenceSpec {
    pub opcode: u16,
    #[schemars(length(min = 1))]
    pub name: String,
}

impl StandardLibrarySpec {
    pub(crate) fn current() -> Self {
        catalog::current()
    }

    pub fn validate(
        &self,
        domain_opset: &super::DomainOpsetSpec,
    ) -> Result<(), VocabularyValidationError> {
        validation::validate(self, domain_opset)
    }
}

impl Default for StandardLibrarySpec {
    fn default() -> Self {
        Self::current()
    }
}

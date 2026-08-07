use std::collections::BTreeMap;

use super::super::{FunctionDefinition, FunctionId, FunctionParameter, ValueType};
use super::function_id;

#[derive(Debug, Clone)]
pub(super) struct FunctionSignature {
    id: FunctionId,
    name: String,
    parameters: Vec<FunctionParameter>,
    return_type: ValueType,
}

pub(super) type FunctionSignatures = BTreeMap<String, FunctionSignature>;

impl FunctionSignature {
    pub(super) fn new(
        id: FunctionId,
        name: String,
        parameters: Vec<FunctionParameter>,
        return_type: ValueType,
    ) -> Self {
        Self {
            id,
            name,
            parameters,
            return_type,
        }
    }

    pub(super) fn from_definition(definition: &FunctionDefinition) -> Self {
        Self::new(
            function_id::identity(definition),
            definition.name.clone(),
            definition.parameters.clone(),
            definition.return_type.clone(),
        )
    }

    pub(super) const fn id(&self) -> FunctionId {
        self.id
    }

    pub(super) fn name(&self) -> &str {
        &self.name
    }

    pub(super) fn parameters(&self) -> &[FunctionParameter] {
        &self.parameters
    }

    pub(super) const fn return_type(&self) -> &ValueType {
        &self.return_type
    }
}

pub(super) fn declarations(definitions: &[FunctionDefinition]) -> FunctionSignatures {
    definitions
        .iter()
        .map(|definition| {
            let signature = FunctionSignature::from_definition(definition);
            (definition.name.clone(), signature)
        })
        .collect()
}

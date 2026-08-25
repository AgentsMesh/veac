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
        let id = function_id::identity(definition);
        let mut parameters = definition.parameters.clone();
        for (slot, parameter) in parameters.iter_mut().enumerate() {
            parameter.bind_default(id, slot);
        }
        Self::new(
            id,
            definition.name.clone(),
            parameters,
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

pub(super) fn method(
    signature: &crate::program::MethodSignature,
    display_name: String,
) -> FunctionSignature {
    let mut parameters = signature.parameters_with_receiver().to_vec();
    for (slot, parameter) in parameters.iter_mut().enumerate().skip(1) {
        parameter.bind_default(signature.function_id(), slot);
    }
    FunctionSignature::new(
        signature.function_id(),
        display_name,
        parameters,
        signature.return_type().clone(),
    )
}

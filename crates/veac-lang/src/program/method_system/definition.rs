use std::sync::Arc;

use crate::program::expression::{FunctionId, FunctionOrigin, FunctionParameter, ValueType};
use crate::program::TypeRef;

use super::identity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodVisibility {
    Private,
    Exported,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSignature {
    receiver: TypeRef,
    name: Arc<str>,
    function_id: FunctionId,
    parameters: Arc<[FunctionParameter]>,
    return_type: ValueType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodBody {
    source: Arc<str>,
    origin: FunctionOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDefinition {
    signature: MethodSignature,
    owner_source: Arc<str>,
    visibility: MethodVisibility,
    body: Option<MethodBody>,
}

impl MethodSignature {
    pub fn new(
        receiver: TypeRef,
        name: impl Into<Arc<str>>,
        explicit_parameters: impl Into<Arc<[FunctionParameter]>>,
        return_type: ValueType,
    ) -> Self {
        let name = name.into();
        let explicit_parameters = explicit_parameters.into();
        let mut parameters = Vec::with_capacity(explicit_parameters.len() + 1);
        parameters.push(FunctionParameter::new(
            "self",
            ValueType::nominal(receiver.clone()),
        ));
        parameters.extend(explicit_parameters.iter().cloned());
        let provisional: Arc<[FunctionParameter]> = parameters.clone().into();
        let function_id = identity::derive(receiver.id(), &name, &provisional, &return_type);
        for (slot, parameter) in parameters.iter_mut().enumerate().skip(1) {
            parameter.bind_default(function_id, slot);
        }
        let parameters: Arc<[FunctionParameter]> = parameters.into();
        Self {
            receiver,
            name,
            function_id,
            parameters,
            return_type,
        }
    }

    pub const fn receiver(&self) -> &TypeRef {
        &self.receiver
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub const fn function_id(&self) -> FunctionId {
        self.function_id
    }

    pub fn parameters_with_receiver(&self) -> &[FunctionParameter] {
        &self.parameters
    }

    pub fn explicit_parameters(&self) -> &[FunctionParameter] {
        &self.parameters[1..]
    }

    pub const fn return_type(&self) -> &ValueType {
        &self.return_type
    }

    pub(crate) fn expected_function_id(&self) -> FunctionId {
        identity::derive(
            self.receiver.id(),
            &self.name,
            &self.parameters,
            &self.return_type,
        )
    }
}

impl MethodBody {
    pub fn new(source: impl Into<Arc<str>>, origin: FunctionOrigin) -> Self {
        Self {
            source: source.into(),
            origin,
        }
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub const fn origin(&self) -> &FunctionOrigin {
        &self.origin
    }
}

impl MethodDefinition {
    pub fn new(
        signature: MethodSignature,
        owner_source: impl Into<Arc<str>>,
        visibility: MethodVisibility,
    ) -> Self {
        Self {
            signature,
            owner_source: owner_source.into(),
            visibility,
            body: None,
        }
    }

    pub fn with_body(mut self, body: MethodBody) -> Self {
        self.body = Some(body);
        self
    }

    pub const fn signature(&self) -> &MethodSignature {
        &self.signature
    }

    pub fn owner_source(&self) -> &str {
        &self.owner_source
    }

    pub const fn visibility(&self) -> MethodVisibility {
        self.visibility
    }

    pub const fn body(&self) -> Option<&MethodBody> {
        self.body.as_ref()
    }

    pub(crate) fn exported_interface(&self) -> Self {
        let mut value = self.clone();
        value.body = None;
        value
    }
}

#[cfg(test)]
pub(super) mod tests_support {
    use super::MethodDefinition;
    use crate::program::expression::FunctionId;

    pub(in crate::program::method_system) fn replace_function_id(
        value: &mut MethodDefinition,
        id: FunctionId,
    ) {
        value.signature.function_id = id;
    }
}

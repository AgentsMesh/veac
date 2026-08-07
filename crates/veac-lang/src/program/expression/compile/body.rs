use super::super::hir::TypedExpression;
use super::super::{ExpressionError, FunctionId, FunctionOrigin, FunctionParameter, ValueType};

pub(super) struct ResolvedBody {
    id: FunctionId,
    name: String,
    parameters: Vec<FunctionParameter>,
    return_type: ValueType,
    source: String,
    origin: Option<FunctionOrigin>,
    visible: bool,
    typed: TypedExpression,
}

impl ResolvedBody {
    pub(super) fn new(
        signature: &super::signature::FunctionSignature,
        source: &str,
        origin: Option<&FunctionOrigin>,
        visible: bool,
        typed: TypedExpression,
    ) -> Self {
        Self {
            id: signature.id(),
            name: signature.name().to_owned(),
            parameters: signature.parameters().to_vec(),
            return_type: signature.return_type().clone(),
            source: source.to_owned(),
            origin: origin.cloned(),
            visible,
            typed,
        }
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

    pub(super) fn source(&self) -> &str {
        &self.source
    }

    pub(super) const fn origin(&self) -> Option<&FunctionOrigin> {
        self.origin.as_ref()
    }

    pub(super) const fn visible(&self) -> bool {
        self.visible
    }

    pub(super) const fn typed(&self) -> &TypedExpression {
        &self.typed
    }

    pub(super) fn decorate(&self, error: ExpressionError) -> ExpressionError {
        error.in_runtime_function(&self.name, self.origin.as_ref())
    }
}

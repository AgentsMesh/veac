use super::super::hir::TypedExpression;
use super::super::{ExpressionError, FunctionId, FunctionOrigin, FunctionParameter, ValueType};

#[derive(Debug, Clone)]
pub(crate) struct ResolvedBody {
    id: FunctionId,
    name: String,
    parameters: Vec<FunctionParameter>,
    return_type: ValueType,
    source: String,
    origin: Option<FunctionOrigin>,
    visible: bool,
    requires_pure: bool,
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
            origin: origin.map(FunctionOrigin::detached),
            visible,
            requires_pure: false,
            typed,
        }
    }

    pub(crate) const fn id(&self) -> FunctionId {
        self.id
    }

    pub(crate) fn name(&self) -> &str {
        &self.name
    }

    pub(crate) fn parameters(&self) -> &[FunctionParameter] {
        &self.parameters
    }

    pub(crate) const fn return_type(&self) -> &ValueType {
        &self.return_type
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) const fn origin(&self) -> Option<&FunctionOrigin> {
        self.origin.as_ref()
    }

    pub(crate) const fn visible(&self) -> bool {
        self.visible
    }

    pub(crate) fn requiring_pure(mut self) -> Self {
        self.requires_pure = true;
        self
    }

    pub(crate) const fn requires_pure(&self) -> bool {
        self.requires_pure
    }

    pub(super) const fn typed(&self) -> &TypedExpression {
        &self.typed
    }

    pub(super) fn decorate(&self, error: ExpressionError) -> ExpressionError {
        error.in_runtime_function(&self.name, self.origin.as_ref())
    }
}

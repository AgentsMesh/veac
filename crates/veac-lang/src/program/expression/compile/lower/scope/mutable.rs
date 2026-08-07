use super::{model, Lowerer};
use crate::program::expression::hir::MutableLocalId;
use crate::program::expression::{ExpressionError, ValueType};

impl Lowerer<'_> {
    pub(in crate::program::expression::compile::lower) fn mutable_target(
        &self,
        name: &str,
        span: std::ops::Range<usize>,
    ) -> Result<(MutableLocalId, ValueType), ExpressionError> {
        let Some(binding) = self.lexical(name) else {
            return Err(ExpressionError::new(
                "EXPRESSION_MUTABLE_TARGET",
                format!("unknown mutable local `{name}`"),
                span,
            ));
        };
        let model::BindingKind::Mutable(id) = binding.kind else {
            return Err(ExpressionError::new(
                "EXPRESSION_MUTABLE_TARGET",
                format!("`{name}` is immutable; only `var` bindings can be assigned"),
                span,
            ));
        };
        if binding.owner < self.closures.len() {
            return Err(ExpressionError::new(
                "EXPRESSION_MUTABLE_CAPTURE",
                "closures cannot assign a captured mutable local",
                span,
            ));
        }
        Ok((id, binding.value_type))
    }
}

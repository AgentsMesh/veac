use super::Lowerer;
use crate::program::expression::ast::TypeAnnotation;
use crate::program::expression::{ExpressionError, ValueType};

impl Lowerer<'_> {
    pub(super) fn resolve_annotation(
        &self,
        annotation: &TypeAnnotation,
    ) -> Result<ValueType, ExpressionError> {
        annotation
            .syntax
            .resolve(&|name| self.types.resolve(name).cloned())
            .map_err(|error| {
                let span = error.span();
                ExpressionError::new(
                    "EXPRESSION_TYPE_SYNTAX",
                    format!("{}: {}", error.code(), error.message()),
                    span.start..span.end,
                )
            })
    }
}

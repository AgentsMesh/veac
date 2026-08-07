use super::Lowerer;
use crate::program::expression::ast::Expression;
use crate::program::expression::ExpressionError;

impl Lowerer<'_> {
    pub(in crate::program::expression::compile::lower) fn reject_self_capture(
        &self,
        name: &str,
        expression: &Expression,
    ) -> Result<(), ExpressionError> {
        if self
            .initializers
            .iter()
            .rev()
            .any(|value| value.name == name && value.owner < self.closures.len())
        {
            Err(ExpressionError::new(
                "EXPRESSION_CLOSURE_SELF_CAPTURE",
                format!("closure cannot capture its initializing binding `{name}`"),
                expression.span.clone(),
            ))
        } else {
            Ok(())
        }
    }
}

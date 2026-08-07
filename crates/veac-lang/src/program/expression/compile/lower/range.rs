use super::Lowerer;
use crate::program::expression::ast::Expression;
use crate::program::expression::hir::{TypedNode, TypedNodeKind};
use crate::program::expression::{ExpressionError, PrimitiveType, ValueType};

impl Lowerer<'_> {
    pub(super) fn range(
        &mut self,
        start: &Expression,
        end: &Expression,
        step: Option<&Expression>,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let start = self.range_operand(start, "start")?;
        let end = self.range_operand(end, "end")?;
        let step = step
            .map(|value| self.range_operand(value, "step").map(Box::new))
            .transpose()?;
        let value_type = ValueType::range(PrimitiveType::Integer.into()).map_err(|error| {
            ExpressionError::new("EXPRESSION_TYPE", error.message(), start.span.clone())
        })?;
        Ok((
            TypedNodeKind::Range {
                start: Box::new(start),
                end: Box::new(end),
                step,
            },
            value_type,
        ))
    }

    fn range_operand(
        &mut self,
        expression: &Expression,
        role: &str,
    ) -> Result<TypedNode, ExpressionError> {
        let value = self.lower(expression)?;
        if value.value_type.as_primitive() == Some(PrimitiveType::Integer) {
            Ok(value)
        } else {
            Err(ExpressionError::new(
                "EXPRESSION_RANGE_TYPE",
                format!("range {role} requires int, found {}", value.value_type),
                value.span,
            ))
        }
    }
}

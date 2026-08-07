use super::Lowerer;
use crate::program::expression::ast::{BinaryOperator, Expression};
use crate::program::expression::hir::{TypedNode, TypedNodeKind};
use crate::program::expression::{ExpressionError, ValueType};

impl Lowerer<'_> {
    pub(super) fn binary(
        &mut self,
        operator: BinaryOperator,
        left: &Expression,
        right: &Expression,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let (left, right) = if equality(operator) {
            self.equality_operands(left, right)?
        } else {
            (self.lower(left)?, self.lower(right)?)
        };
        let value_type = super::super::typing::binary(
            operator,
            &left.value_type,
            &right.value_type,
            &self.types,
            expression.span.clone(),
        )?;
        Ok((
            TypedNodeKind::Binary {
                operator,
                left: Box::new(left),
                right: Box::new(right),
            },
            value_type,
        ))
    }

    fn equality_operands(
        &mut self,
        left: &Expression,
        right: &Expression,
    ) -> Result<(TypedNode, TypedNode), ExpressionError> {
        let checkpoint = self.checkpoint();
        match self.lower(left) {
            Ok(left) => {
                let right = self.lower_context(right, Some(&left.value_type))?;
                Ok((left, right))
            }
            Err(error) if needs_context(&error) => {
                self.rollback(&checkpoint);
                let inferred = self.lower(right)?.value_type;
                self.rollback(&checkpoint);
                let left = self.lower_context(left, Some(&inferred))?;
                let right = self.lower_context(right, Some(&left.value_type))?;
                Ok((left, right))
            }
            Err(error) => Err(error),
        }
    }
}

fn equality(operator: BinaryOperator) -> bool {
    matches!(operator, BinaryOperator::Equal | BinaryOperator::NotEqual)
}

pub(super) fn needs_context(error: &ExpressionError) -> bool {
    error.code() == "EXPRESSION_COLLECTION_TYPE_CONTEXT"
}

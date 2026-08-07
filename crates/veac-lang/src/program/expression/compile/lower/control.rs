use super::scope::{Initializer, ScopeBinding};
use super::Lowerer;
use crate::program::expression::ast::{Block, Expression};
use crate::program::expression::hir::{TypedBlock, TypedNodeKind};
use crate::program::expression::{ExpressionError, ValueType};

mod statement;

impl Lowerer<'_> {
    pub(super) fn block(
        &mut self,
        block: &Block,
        expected: Option<&ValueType>,
    ) -> Result<TypedBlock, ExpressionError> {
        self.scopes.push(Default::default());
        let result = self.block_contents(block, expected);
        self.scopes.pop();
        result
    }

    fn block_contents(
        &mut self,
        block: &Block,
        expected: Option<&ValueType>,
    ) -> Result<TypedBlock, ExpressionError> {
        let mut statements = Vec::with_capacity(block.statements.len());
        for statement in &block.statements {
            statements.push(self.statement(statement)?);
        }
        let result = Box::new(self.lower_context(&block.result, expected)?);
        Ok(TypedBlock { statements, result })
    }

    pub(super) fn conditional(
        &mut self,
        condition: &Expression,
        then_branch: &Block,
        else_branch: &Block,
        expected: Option<&ValueType>,
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let condition = self.lower(condition)?;
        if condition.value_type.as_primitive()
            != Some(crate::program::expression::PrimitiveType::Boolean)
        {
            return Err(ExpressionError::new(
                "EXPRESSION_TYPE",
                format!("if condition requires bool, found {}", condition.value_type),
                condition.span.clone(),
            ));
        }
        let (then_branch, else_branch) = self.branches(then_branch, else_branch, expected)?;
        if then_branch.result.value_type != else_branch.result.value_type {
            return Err(ExpressionError::new(
                "EXPRESSION_TYPE",
                format!(
                    "if branches must have one type, found {} and {}",
                    then_branch.result.value_type, else_branch.result.value_type
                ),
                expression.span.clone(),
            ));
        }
        let value_type = then_branch.result.value_type.clone();
        Ok((
            TypedNodeKind::If {
                condition: Box::new(condition),
                then_branch,
                else_branch,
            },
            value_type,
        ))
    }

    fn branches(
        &mut self,
        then_branch: &Block,
        else_branch: &Block,
        expected: Option<&ValueType>,
    ) -> Result<(TypedBlock, TypedBlock), ExpressionError> {
        let checkpoint = self.checkpoint();
        match self.block(then_branch, expected) {
            Ok(then_branch) => {
                let branch_type = expected.unwrap_or(&then_branch.result.value_type);
                let else_branch = self.block(else_branch, Some(branch_type))?;
                Ok((then_branch, else_branch))
            }
            Err(error) if expected.is_none() && super::binary::needs_context(&error) => {
                self.rollback(&checkpoint);
                let inferred = self.block(else_branch, None)?.result.value_type;
                self.rollback(&checkpoint);
                let then_branch = self.block(then_branch, Some(&inferred))?;
                let else_branch = self.block(else_branch, Some(&then_branch.result.value_type))?;
                Ok((then_branch, else_branch))
            }
            Err(error) => Err(error),
        }
    }
}

pub(super) fn require_binding_type(
    name: &str,
    value: &crate::program::expression::hir::TypedNode,
    expected: &ValueType,
) -> Result<(), ExpressionError> {
    if &value.value_type == expected {
        Ok(())
    } else {
        Err(ExpressionError::new(
            "EXPRESSION_TYPE",
            format!(
                "binding `{name}` declares {expected}, but its value has type {}",
                value.value_type
            ),
            value.span.clone(),
        ))
    }
}

use super::super::Lowerer;
use crate::program::expression::ast::{CallArgument, Expression};
use crate::program::expression::hir::{TypedNode, TypedNodeKind};
use crate::program::expression::{ExpressionError, ValueType, ValueTypeKind};

impl Lowerer<'_> {
    pub(in crate::program::expression::compile::lower) fn invoke(
        &mut self,
        callee: TypedNode,
        arguments: &[CallArgument],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        super::reject_named("function value", arguments, expression)?;
        let ValueTypeKind::Function {
            parameters, result, ..
        } = callee.value_type.kind()
        else {
            return Err(ExpressionError::new(
                "EXPRESSION_CALL_CALLEE_TYPE",
                format!(
                    "call requires a function value, found {}",
                    callee.value_type
                ),
                callee.span.clone(),
            ));
        };
        let parameters = parameters.to_vec();
        let result = result.clone();
        super::super::typing::require_arity(
            "function value",
            arguments.len(),
            parameters.len(),
            expression.span.clone(),
        )?;
        let arguments = arguments
            .iter()
            .zip(&parameters)
            .enumerate()
            .map(|(index, (argument, expected))| {
                let value = self.lower_context(&argument.value, Some(expected))?;
                check_value_argument(index, &value, expected, &argument.value)?;
                Ok(value)
            })
            .collect::<Result<Vec<_>, ExpressionError>>()?;
        Ok((
            TypedNodeKind::Invoke {
                callee: Box::new(callee),
                arguments,
            },
            result,
        ))
    }
}

pub(in crate::program::expression::compile::lower) fn check_value_argument(
    index: usize,
    value: &TypedNode,
    expected: &ValueType,
    expression: &Expression,
) -> Result<(), ExpressionError> {
    if &value.value_type == expected {
        return Ok(());
    }
    Err(ExpressionError::new(
        "EXPRESSION_CALL_ARGUMENT_TYPE",
        format!(
            "argument {} expects {expected}, found {}",
            index + 1,
            value.value_type
        ),
        expression.span.clone(),
    ))
}

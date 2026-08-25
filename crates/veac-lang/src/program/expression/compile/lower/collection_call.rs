use super::Lowerer;
use crate::program::expression::ast::{CallArgument, Expression};
use crate::program::expression::hir::TypedNodeKind;
use crate::program::expression::{CollectionOperation, ExpressionError, PrimitiveType, ValueType};

mod inference;
pub(super) mod typing;

use typing::callback_result;
pub(super) use typing::{iterable_element, list_type};

impl Lowerer<'_> {
    pub(super) fn collection_call(
        &mut self,
        operation: CollectionOperation,
        arguments: &[CallArgument],
        expression: &Expression,
    ) -> Result<(TypedNodeKind, ValueType), ExpressionError> {
        let arity = if operation == CollectionOperation::Fold {
            3
        } else {
            2
        };
        super::typing::require_arity(
            operation.as_str(),
            arguments.len(),
            arity,
            expression.span.clone(),
        )?;
        let input = self.collection_input(operation, arguments)?;
        let element = iterable_element(&input.value_type, &arguments[0].value)?;
        let mut lowered = vec![input];
        let value_type = match operation {
            CollectionOperation::Map => {
                let callback = self.lower(&arguments[1].value)?;
                let result =
                    callback_result(operation, &callback, &[element], None, &arguments[1].value)?;
                lowered.push(callback);
                list_type(result, expression)?
            }
            CollectionOperation::Filter => {
                let callback = self.lower(&arguments[1].value)?;
                callback_result(
                    operation,
                    &callback,
                    std::slice::from_ref(&element),
                    Some(&PrimitiveType::Boolean.into()),
                    &arguments[1].value,
                )?;
                lowered.push(callback);
                list_type(element, expression)?
            }
            CollectionOperation::Fold => {
                let initial = self.fold_initial(arguments)?;
                let accumulator = initial.value_type.clone();
                let callback = self.lower(&arguments[2].value)?;
                callback_result(
                    operation,
                    &callback,
                    &[accumulator.clone(), element],
                    Some(&accumulator),
                    &arguments[2].value,
                )?;
                lowered.extend([initial, callback]);
                accumulator
            }
        };
        Ok((
            TypedNodeKind::Collection {
                operation,
                arguments: lowered,
            },
            value_type,
        ))
    }
}

#[cfg(test)]
#[path = "collection_call/tests.rs"]
mod tests;

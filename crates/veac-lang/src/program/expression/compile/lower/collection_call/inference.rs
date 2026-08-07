use super::super::Lowerer;
use super::typing::{callback_parameter, list_type, type_error};
use crate::program::expression::ast::{Expression, ExpressionKind};
use crate::program::expression::hir::TypedNode;
use crate::program::expression::{CollectionOperation, ExpressionError, ValueType, ValueTypeKind};

impl Lowerer<'_> {
    pub(super) fn collection_input(
        &mut self,
        operation: CollectionOperation,
        arguments: &[Expression],
    ) -> Result<TypedNode, ExpressionError> {
        let checkpoint = self.checkpoint();
        match self.lower(&arguments[0]) {
            Ok(input) => Ok(input),
            Err(error) if is_context_error(&error) => {
                self.rollback(&checkpoint);
                let (callback, index, arity) = match operation {
                    CollectionOperation::Map | CollectionOperation::Filter => (1, 0, 1),
                    CollectionOperation::Fold => (2, 1, 2),
                };
                let element = self.probe_parameter(&arguments[callback], index, arity)?;
                let expected = expected_container(&arguments[0], &element)?;
                self.lower_context(&arguments[0], Some(&expected))
            }
            Err(error) => Err(error),
        }
    }

    pub(super) fn fold_initial(
        &mut self,
        arguments: &[Expression],
    ) -> Result<TypedNode, ExpressionError> {
        let checkpoint = self.checkpoint();
        match self.lower(&arguments[1]) {
            Ok(initial) => Ok(initial),
            Err(error) if is_context_error(&error) => {
                self.rollback(&checkpoint);
                let accumulator = self.probe_parameter(&arguments[2], 0, 2)?;
                self.lower_context(&arguments[1], Some(&accumulator))
            }
            Err(error) => Err(error),
        }
    }

    fn probe_parameter(
        &mut self,
        callback: &Expression,
        index: usize,
        arity: usize,
    ) -> Result<ValueType, ExpressionError> {
        let checkpoint = self.checkpoint();
        let probe = self.lower(callback);
        self.rollback(&checkpoint);
        callback_parameter(&probe?, index, arity, callback)
    }
}

fn expected_container(
    input: &Expression,
    element: &ValueType,
) -> Result<ValueType, ExpressionError> {
    match &input.kind {
        ExpressionKind::List(values) if values.is_empty() => list_type(element.clone(), input),
        ExpressionKind::Map(entries) if entries.is_empty() => expected_map(element, input),
        _ => Err(type_error(
            "empty iterable requires a typed list or map literal",
            input,
        )),
    }
}

fn expected_map(element: &ValueType, input: &Expression) -> Result<ValueType, ExpressionError> {
    let ValueTypeKind::Tuple(parts) = element.kind() else {
        return Err(type_error(
            "map callback requires a (key, value) parameter",
            input,
        ));
    };
    if parts.len() != 2 {
        return Err(type_error(
            "map callback requires a (key, value) parameter",
            input,
        ));
    }
    ValueType::map(parts[0].clone(), parts[1].clone())
        .map_err(|error| type_error(error.message(), input))
}

fn is_context_error(error: &ExpressionError) -> bool {
    error.code() == "EXPRESSION_COLLECTION_TYPE_CONTEXT"
}

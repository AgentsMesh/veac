use std::ops::Range;

use super::super::super::{BuiltinFunction, ExpressionError, ValueType};
use super::require_arity;

pub(super) fn check(
    function: BuiltinFunction,
    arguments: &[ValueType],
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    let expected = function.arity();
    require_arity(function.as_str(), arguments.len(), expected, span.clone())?;
    function
        .result_type(arguments)
        .map_err(|message| argument_error(message, span))
}

fn argument_error(message: impl Into<String>, span: Range<usize>) -> ExpressionError {
    ExpressionError::new("EXPRESSION_CALL_ARGUMENT_TYPE", message, span)
}

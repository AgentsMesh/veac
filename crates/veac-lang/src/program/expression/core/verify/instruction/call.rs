use std::ops::Range;

use super::super::{definitions::Definitions, error, type_table, types};
use crate::program::expression::core::{CoreCallTarget, CoreProgram, FunctionRegistry, ValueId};
use crate::program::expression::{ExpressionError, ValueType};

pub(super) fn verify(
    target: CoreCallTarget,
    arguments: &[ValueId],
    program: &CoreProgram,
    definitions: &Definitions,
    registry: &FunctionRegistry,
    span: Range<usize>,
) -> Result<ValueType, ExpressionError> {
    let arguments = arguments
        .iter()
        .map(|id| value_type(program, definitions, *id, span.clone()).cloned())
        .collect::<Result<Vec<_>, _>>()?;
    match target {
        CoreCallTarget::Builtin(function) => types::builtin(function, &arguments, span),
        CoreCallTarget::User(id) => {
            let function = registry
                .get(id)
                .ok_or_else(|| error(format!("unknown FunctionId {id}"), span.clone()))?;
            types::user_call(function, &arguments, span)?;
            Ok(function.return_type().clone())
        }
    }
}

fn value_type<'a>(
    program: &'a CoreProgram,
    definitions: &Definitions,
    id: ValueId,
    span: Range<usize>,
) -> Result<&'a ValueType, ExpressionError> {
    type_table::value(program, definitions.type_id(id), span)
}

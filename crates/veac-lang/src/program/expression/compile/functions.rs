use super::super::{ExpressionContext, ExpressionError, FunctionDefinition, FunctionMap};
use super::query;

pub(super) mod prepare;
pub(super) mod validation;

pub(super) fn compile(
    context: &ExpressionContext,
    definitions: &[FunctionDefinition],
) -> Result<FunctionMap, ExpressionError> {
    compile_bounded(context, definitions, usize::MAX)
}

pub(super) fn compile_bounded(
    context: &ExpressionContext,
    definitions: &[FunctionDefinition],
    retained_limit: usize,
) -> Result<FunctionMap, ExpressionError> {
    let batch = query::prepare(context, definitions)?;
    query::lower(context, &batch, retained_limit).map(|batch| batch.functions().clone())
}

#[cfg(test)]
mod tests;

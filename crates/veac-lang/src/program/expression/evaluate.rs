use super::{
    CompiledExpression, Environment, ExecutionBudget, ExpressionContext, ExpressionError, Value,
    ValueLookup,
};
use crate::program::syntax_document::{SyntaxDocument, SyntaxSlice};

pub(crate) fn lookup_slice_with_budget(
    document: &SyntaxDocument,
    slice: &SyntaxSlice,
    environment: &dyn ValueLookup,
    context: &ExpressionContext,
    execution: &ExecutionBudget,
) -> Result<Value, ExpressionError> {
    let expression =
        super::compile::compile_slice_with_values(document, slice, environment, context)?;
    super::runtime::execute(&expression, environment, execution)
}

pub fn source(source: &str, environment: &Environment) -> Result<Value, ExpressionError> {
    in_context(source, environment, &ExpressionContext::empty())
}

pub(crate) fn lookup(
    source: &str,
    environment: &dyn ValueLookup,
    context: &ExpressionContext,
) -> Result<Value, ExpressionError> {
    lookup_with_budget(source, environment, context, &ExecutionBudget::default())
}

pub(crate) fn lookup_with_budget(
    source: &str,
    environment: &dyn ValueLookup,
    context: &ExpressionContext,
    execution: &ExecutionBudget,
) -> Result<Value, ExpressionError> {
    let expression = super::compile::compile_with_values(source, environment, context)?;
    super::runtime::execute(&expression, environment, execution)
}

pub fn in_context(
    source: &str,
    environment: &Environment,
    context: &ExpressionContext,
) -> Result<Value, ExpressionError> {
    lookup(source, environment, context)
}

pub fn compiled(
    expression: &CompiledExpression,
    environment: &Environment,
) -> Result<Value, ExpressionError> {
    super::runtime::execute(expression, environment, &ExecutionBudget::default())
}

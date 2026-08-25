use super::{functions, ExpressionContext, ExpressionError, FunctionDefinition};

pub(crate) fn compile_functions_bounded(
    context: &ExpressionContext,
    definitions: &[FunctionDefinition],
    retained_limit: usize,
) -> Result<ExpressionContext, ExpressionError> {
    let functions = functions::compile_bounded(context, definitions, retained_limit)?;
    Ok(context.clone().with_functions(functions))
}

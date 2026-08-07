use std::sync::Arc;

use super::super::{
    CompiledFunction, ExpressionContext, ExpressionError, FunctionDefinition, FunctionMap,
};
use super::body::ResolvedBody;
use super::{core, function_id, graph, signature};

mod prepare;
mod validation;

use validation::validate_definitions;

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
    validate_definitions(context, definitions)?;
    let signatures = signature::declarations(definitions);
    let bodies = prepare::all(context, definitions, &signatures)?;
    let order = graph::order(context.functions().registry(), &bodies)?;
    let mut functions = context.functions().clone();
    let mut retained = 0usize;
    for index in order {
        let body = &bodies[index];
        let compiled = compile_body(body, &functions, context)?;
        retained = charge_retained(retained, &compiled, body, retained_limit)?;
        if body.visible() {
            functions.insert(Arc::new(compiled));
        } else {
            functions.insert_hidden(Arc::new(compiled));
        }
    }
    Ok(functions)
}

fn compile_body(
    body: &ResolvedBody,
    functions: &FunctionMap,
    context: &ExpressionContext,
) -> Result<CompiledFunction, ExpressionError> {
    let parameter_types = body
        .parameters()
        .iter()
        .map(|parameter| parameter.value_type.clone())
        .collect::<Vec<_>>();
    let program = core::lower(
        body.typed(),
        functions.registry(),
        &|_| false,
        &|_| None,
        &|name| {
            let id = context.build_input(name).map_or_else(
                || super::super::CoreBuildInputId::for_symbol(name),
                |input| input.id(),
            );
            super::super::CoreInputIdentity::Build(id)
        },
        &parameter_types,
    );
    let digest = function_id::content_body(body.source(), &program, functions.registry());
    let verified = super::super::core::verify(program, functions.registry(), body.parameters())
        .map_err(|error| body.decorate(error))?;
    Ok(CompiledFunction::new(
        body.id(),
        digest,
        body.name().to_owned(),
        body.parameters().to_vec(),
        body.return_type().clone(),
        body.origin().cloned(),
        verified,
    ))
}

fn charge_retained(
    retained: usize,
    function: &CompiledFunction,
    body: &ResolvedBody,
    limit: usize,
) -> Result<usize, ExpressionError> {
    let next = function
        .retained_bytes()
        .and_then(|bytes| retained.checked_add(bytes));
    match next {
        Some(next) if next <= limit => Ok(next),
        _ => Err(body.decorate(ExpressionError::new(
            "EXPRESSION_RETAINED_LIMIT",
            "compiled function Core exceeds the retained symbol storage budget",
            body.typed().root.span.clone(),
        ))),
    }
}

#[cfg(test)]
mod tests;

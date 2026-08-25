use std::sync::Arc;

use super::super::{ExpressionContext, ExpressionError, FunctionDefinition, FunctionMap};
use super::body::ResolvedBody;
use super::{core, function_id, graph, signature};

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub(crate) struct TypedFunctionBatch {
    bodies: Vec<ResolvedBody>,
    order: Vec<usize>,
}

#[derive(Debug, Clone)]
pub(crate) struct CompiledFunctionBatch {
    functions: FunctionMap,
    retained_bytes: usize,
}

impl TypedFunctionBatch {
    pub(crate) fn bodies(&self) -> &[ResolvedBody] {
        &self.bodies
    }

    pub(crate) fn order(&self) -> &[usize] {
        &self.order
    }
}

impl CompiledFunctionBatch {
    pub(crate) fn functions(&self) -> &FunctionMap {
        &self.functions
    }

    pub(crate) fn retained_bytes(&self) -> usize {
        self.retained_bytes
    }
}

pub(crate) fn prepare(
    context: &ExpressionContext,
    definitions: &[FunctionDefinition],
) -> Result<TypedFunctionBatch, ExpressionError> {
    super::functions::validation::validate_definitions(context, definitions)?;
    let signatures = signature::declarations(definitions);
    let bodies = super::functions::prepare::all(context, definitions, &signatures)?;
    let order = graph::order(context.functions().registry(), &bodies)?;
    Ok(TypedFunctionBatch { bodies, order })
}

pub(crate) fn lower(
    context: &ExpressionContext,
    batch: &TypedFunctionBatch,
    retained_limit: usize,
) -> Result<CompiledFunctionBatch, ExpressionError> {
    let mut functions = context.functions().clone();
    let mut retained = 0usize;
    for &index in batch.order() {
        let body = &batch.bodies()[index];
        let compiled = lower_body(body, &functions, context)?;
        retained = charge(retained, &compiled, body, retained_limit)?;
        if body.visible() {
            functions.insert(Arc::new(compiled));
        } else {
            functions.insert_hidden(Arc::new(compiled));
        }
    }
    Ok(CompiledFunctionBatch {
        functions,
        retained_bytes: retained,
    })
}

fn lower_body(
    body: &ResolvedBody,
    functions: &FunctionMap,
    context: &ExpressionContext,
) -> Result<super::super::CompiledFunction, ExpressionError> {
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
    let digest = function_id::content_body(
        body.source(),
        &program,
        body.parameters(),
        functions.registry(),
    );
    let verified = super::super::core::verify(program, functions.registry(), body.parameters())
        .map_err(|error| body.decorate(error))?;
    let compiled = super::super::CompiledFunction::new(
        body.id(),
        digest,
        body.name().to_owned(),
        body.parameters().to_vec(),
        body.return_type().clone(),
        body.origin().cloned(),
        verified,
    );
    if body.requires_pure() {
        super::defaults::require_pure(compiled.summary(), body)?;
    }
    Ok(compiled)
}

fn charge(
    retained: usize,
    function: &super::super::CompiledFunction,
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

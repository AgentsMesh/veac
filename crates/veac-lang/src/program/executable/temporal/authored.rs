use std::collections::BTreeSet;

use crate::program::diagnostic::{Diagnostic, Diagnostics};
use crate::program::expression::{compile_temporal_slice, ExpressionContext, TypeEnvironment};
use crate::program::model::{SurfaceFile, TemporalDecl};

use super::ExecutableTemporalLeaf;

mod identity;
mod input;
pub(super) mod sink;

pub(in crate::program::executable) fn compile(
    file: &SurfaceFile,
    context: &ExpressionContext,
) -> Result<Vec<ExecutableTemporalLeaf>, Diagnostics> {
    let mut sinks = BTreeSet::new();
    let mut leaves = Vec::with_capacity(file.temporal.len());
    for declaration in &file.temporal {
        let leaf = one(file, context, declaration).map_err(Diagnostics::one)?;
        if !sinks.insert(leaf.sink().clone()) {
            return Err(Diagnostics::one(Diagnostic::new(
                "PROGRAM_TEMPORAL_SINK_DUPLICATE",
                &file.path,
                "one clip property cannot have more than one authored animation",
                declaration.span,
            )));
        }
        leaves.push(leaf);
    }
    Ok(leaves)
}

fn one(
    file: &SurfaceFile,
    context: &ExpressionContext,
    declaration: &TemporalDecl,
) -> Result<ExecutableTemporalLeaf, Diagnostic> {
    if declaration
        .source
        .as_ref()
        .is_some_and(|source| source.project != declaration.target.project())
    {
        return Err(Diagnostic::new(
            "PROGRAM_TEMPORAL_SOURCE_PROJECT",
            &file.path,
            "a temporal source resource must belong to the target clip project",
            declaration.span,
        ));
    }
    if declaration.source.is_some() && declaration.target.item().is_none() {
        return Err(Diagnostic::new(
            "PROGRAM_TEMPORAL_SOURCE_TARGET",
            &file.path,
            "an apply-owned animation cannot declare an item source clock",
            declaration.span,
        ));
    }
    let sink = sink::lower(&declaration.target, declaration.property)
        .map_err(|message| target(file, declaration, message))?;
    let sequence_id = sink
        .sequence_id()
        .cloned()
        .or_else(|| sink::clip_sequence(&declaration.target).ok())
        .ok_or_else(|| target(file, declaration, "temporal target has no sequence owner"))?;
    let source_id = declaration
        .source
        .as_ref()
        .map(|source| super::super::lower::id::material(&source.segments()));
    let inputs = match sink.item_id() {
        Some(item_id) => input::clip_clocks(item_id, &sequence_id, source_id),
        None => input::sequence_clocks(&sequence_id),
    };
    let expression = compile_temporal_slice(
        &file.syntax,
        &declaration.body.syntax,
        &TypeEnvironment::new(),
        &inputs,
        context,
    );
    let expression = expression_result(expression, file, declaration)?;
    Ok(ExecutableTemporalLeaf::new(
        sink,
        identity::binding(&declaration.target, declaration.property),
        expression,
        identity::request(file, declaration),
    ))
}

fn expression(
    file: &SurfaceFile,
    declaration: &TemporalDecl,
    error: crate::program::expression::ExpressionError,
) -> Diagnostic {
    let relative = error.span();
    let start = declaration.body.span.start.saturating_add(relative.start);
    let end = declaration.body.span.start.saturating_add(relative.end);
    Diagnostic::new(
        "PROGRAM_TEMPORAL_EXPRESSION",
        &file.path,
        error.to_string(),
        crate::authoring::Span {
            start: start.min(declaration.body.span.end),
            end: end.min(declaration.body.span.end),
        },
    )
}

fn target(
    file: &SurfaceFile,
    declaration: &TemporalDecl,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic::new(
        "PROGRAM_TEMPORAL_TARGET",
        &file.path,
        message,
        declaration.span,
    )
}

fn expression_result<T>(
    result: Result<T, crate::program::expression::ExpressionError>,
    file: &SurfaceFile,
    declaration: &TemporalDecl,
) -> Result<T, Diagnostic> {
    match result {
        Ok(value) => Ok(value),
        Err(error) => Err(expression(file, declaration, error)),
    }
}

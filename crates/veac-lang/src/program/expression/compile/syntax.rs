use std::collections::BTreeMap;

use super::super::{
    CompiledExpression, CoreBuildInputId, CoreInputIdentity, CoreTemporalInputIdentity,
    ExpressionContext, ExpressionError, TypeEnvironment, ValueLookup,
};
use crate::program::syntax_document::{SyntaxDocument, SyntaxSlice};

pub(crate) fn compile_expression_slice(
    document: &SyntaxDocument,
    slice: &SyntaxSlice,
    environment: &TypeEnvironment,
    context: &ExpressionContext,
) -> Result<CompiledExpression, ExpressionError> {
    compile(
        document,
        slice,
        &|name| {
            environment.get(name).cloned().or_else(|| {
                context
                    .build_input(name)
                    .map(|input| input.value_type().clone())
            })
        },
        &|_| false,
        &|_| None,
        &|name| CoreInputIdentity::Build(super::build_input_id(context, name)),
        context,
    )
}

pub(crate) fn compile_temporal_slice(
    document: &SyntaxDocument,
    slice: &SyntaxSlice,
    build: &TypeEnvironment,
    inputs: &BTreeMap<String, CoreTemporalInputIdentity>,
    context: &ExpressionContext,
) -> Result<CompiledExpression, ExpressionError> {
    compile(
        document,
        slice,
        &|name| {
            inputs
                .get(name)
                .map(CoreTemporalInputIdentity::value_type)
                .or_else(|| build.get(name).cloned())
                .or_else(|| {
                    context
                        .build_input(name)
                        .map(|input| input.value_type().clone())
                })
        },
        &|_| false,
        &|_| None,
        &|name| {
            inputs.get(name).cloned().map_or_else(
                || CoreInputIdentity::Build(super::build_input_id(context, name)),
                CoreInputIdentity::Temporal,
            )
        },
        context,
    )
}

pub(in crate::program::expression) fn compile_slice_with_values(
    document: &SyntaxDocument,
    slice: &SyntaxSlice,
    environment: &dyn ValueLookup,
    context: &ExpressionContext,
) -> Result<CompiledExpression, ExpressionError> {
    compile(
        document,
        slice,
        &|name| environment.value(name).map(|value| value.value_type()),
        &|name| environment.trusts_function_value(name),
        &|name| environment.callable_input(name),
        &|name| CoreInputIdentity::Build(CoreBuildInputId::for_symbol(name)),
        context,
    )
}

fn compile(
    document: &SyntaxDocument,
    slice: &SyntaxSlice,
    environment: &dyn Fn(&str) -> Option<super::super::ValueType>,
    trusted: &dyn Fn(&str) -> bool,
    callable: &dyn Fn(&str) -> Option<super::super::CoreCallableInput>,
    identity: &dyn Fn(&str) -> CoreInputIdentity,
    context: &ExpressionContext,
) -> Result<CompiledExpression, ExpressionError> {
    let syntax = super::super::cst_adapter::ExpressionSyntax::new(document, slice);
    super::compile_parsed_with_types(
        syntax.parse()?,
        environment,
        trusted,
        callable,
        identity,
        context,
    )
}

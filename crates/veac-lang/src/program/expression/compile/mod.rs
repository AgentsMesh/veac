mod body;
mod core;
mod defaults;
mod function_id;
mod functions;
mod graph;
mod known_type;
mod lower;
mod query;
mod query_key;
mod query_retained;
mod signature;
mod syntax;
mod typing;

#[cfg(test)]
pub(crate) mod test_support;

use std::sync::Arc;

use super::{
    CompiledExpression, CompiledFunction, CoreBuildInputId, CoreCallableInput, CoreInputIdentity,
    CoreTemporalInputIdentity, ExpressionContext, ExpressionError, FunctionDefinition,
    TypeEnvironment, ValueLookup, ValueType,
};
pub(crate) use query::{lower as lower_function_batch, prepare as prepare_function_batch};
pub(crate) use query::{CompiledFunctionBatch, TypedFunctionBatch};
pub(crate) use query_key::FunctionQueryKey;
pub(super) use syntax::compile_slice_with_values;
pub(crate) use syntax::{compile_expression_slice, compile_temporal_slice};

pub fn compile_expression(
    source: &str,
    environment: &TypeEnvironment,
    context: &ExpressionContext,
) -> Result<CompiledExpression, ExpressionError> {
    compile_with_types(
        source,
        &|name| {
            environment.get(name).cloned().or_else(|| {
                context
                    .build_input(name)
                    .map(|input| input.value_type().clone())
            })
        },
        &|_| false,
        &|_| None,
        &|name| CoreInputIdentity::Build(build_input_id(context, name)),
        context,
    )
}

pub fn compile_temporal_expression(
    source: &str,
    build_environment: &TypeEnvironment,
    temporal_inputs: &std::collections::BTreeMap<String, CoreTemporalInputIdentity>,
    context: &ExpressionContext,
) -> Result<CompiledExpression, ExpressionError> {
    compile_with_types(
        source,
        &|name| {
            temporal_inputs
                .get(name)
                .map(CoreTemporalInputIdentity::value_type)
                .or_else(|| build_environment.get(name).cloned())
                .or_else(|| {
                    context
                        .build_input(name)
                        .map(|input| input.value_type().clone())
                })
        },
        &|_| false,
        &|_| None,
        &|name| {
            temporal_inputs.get(name).cloned().map_or_else(
                || CoreInputIdentity::Build(build_input_id(context, name)),
                CoreInputIdentity::Temporal,
            )
        },
        context,
    )
}

pub(super) fn build_input_id(context: &ExpressionContext, name: &str) -> CoreBuildInputId {
    context
        .build_input(name)
        .map_or_else(|| CoreBuildInputId::for_symbol(name), |input| input.id())
}

pub fn compile_function(
    context: &ExpressionContext,
    definition: &FunctionDefinition,
) -> Result<Arc<CompiledFunction>, ExpressionError> {
    let compiled = compile_functions(context, std::slice::from_ref(definition))?;
    Ok(compiled
        .functions()
        .lookup(&definition.name)
        .expect("compiled definition must be present")
        .clone())
}

pub fn compile_functions(
    context: &ExpressionContext,
    definitions: &[FunctionDefinition],
) -> Result<ExpressionContext, ExpressionError> {
    let functions = functions::compile(context, definitions)?;
    Ok(context.clone().with_functions(functions))
}

pub(super) fn compile_with_values(
    source: &str,
    environment: &dyn ValueLookup,
    context: &ExpressionContext,
) -> Result<CompiledExpression, ExpressionError> {
    compile_with_types(
        source,
        &|name| environment.value(name).map(|value| value.value_type()),
        &|name| environment.trusts_function_value(name),
        &|name| environment.callable_input(name),
        &|name| CoreInputIdentity::Build(CoreBuildInputId::for_symbol(name)),
        context,
    )
}

fn compile_with_types(
    source: &str,
    environment: &dyn Fn(&str) -> Option<ValueType>,
    trusted_functions: &dyn Fn(&str) -> bool,
    callable_inputs: &dyn Fn(&str) -> Option<CoreCallableInput>,
    input_identity: &dyn Fn(&str) -> CoreInputIdentity,
    context: &ExpressionContext,
) -> Result<CompiledExpression, ExpressionError> {
    compile_parsed_with_types(
        super::cst_adapter::parse(source, None)?,
        environment,
        trusted_functions,
        callable_inputs,
        input_identity,
        context,
    )
}

pub(super) fn compile_parsed_with_types(
    expression: super::ast::Expression,
    environment: &dyn Fn(&str) -> Option<ValueType>,
    trusted_functions: &dyn Fn(&str) -> bool,
    callable_inputs: &dyn Fn(&str) -> Option<CoreCallableInput>,
    input_identity: &dyn Fn(&str) -> CoreInputIdentity,
    context: &ExpressionContext,
) -> Result<CompiledExpression, ExpressionError> {
    known_type::context(context)?;
    let typed = lower::expression(
        &expression,
        &|name| {
            environment(name).map(|value_type| {
                if trusted_functions(name) {
                    lower::SymbolTarget::TrustedExternal(value_type)
                } else {
                    lower::SymbolTarget::External(value_type)
                }
            })
        },
        context,
    )?;
    let program = core::lower(
        &typed,
        context.functions().registry(),
        trusted_functions,
        callable_inputs,
        input_identity,
        &[],
    );
    let program = super::core::verify_with_input_trust(
        program,
        context.functions().registry(),
        &[],
        trusted_functions,
    )?;
    Ok(CompiledExpression::new(
        program,
        context.functions().registry_arc(),
    ))
}

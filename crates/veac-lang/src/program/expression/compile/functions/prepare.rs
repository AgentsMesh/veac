use std::collections::BTreeMap;

use super::super::super::ast::{Expression, ExpressionKind};
use super::super::super::{
    ExpressionContext, ExpressionError, FunctionDefinition, FunctionOrigin, FunctionParameter,
    ValueType,
};
use super::super::body::ResolvedBody;
use super::super::lower::{self, SymbolTarget};
use super::super::signature::{FunctionSignature, FunctionSignatures};

pub(super) fn all(
    context: &ExpressionContext,
    definitions: &[FunctionDefinition],
    signatures: &FunctionSignatures,
) -> Result<Vec<ResolvedBody>, ExpressionError> {
    let mut output = definitions
        .iter()
        .map(|definition| regular(context, definition, signatures))
        .collect::<Result<Vec<_>, _>>()?;
    output.extend(
        context
            .methods()
            .definitions()
            .filter(|definition| definition.body().is_some())
            .map(|definition| method(context, definition, signatures))
            .collect::<Result<Vec<_>, _>>()?,
    );
    Ok(output)
}

fn regular(
    context: &ExpressionContext,
    definition: &FunctionDefinition,
    signatures: &FunctionSignatures,
) -> Result<ResolvedBody, ExpressionError> {
    let signature = signatures
        .get(&definition.name)
        .expect("validated local signature is registered");
    resolved(
        context,
        signature,
        &definition.body,
        definition.origin.as_ref(),
        true,
        signatures,
    )
}

fn method(
    context: &ExpressionContext,
    definition: &crate::program::MethodDefinition,
    signatures: &FunctionSignatures,
) -> Result<ResolvedBody, ExpressionError> {
    let method = definition.body().expect("filtered authored method body");
    let signature = definition.signature();
    let display_name = format!("{}.{}", signature.receiver(), signature.name());
    let local = FunctionSignature::new(
        signature.function_id(),
        display_name,
        signature.parameters_with_receiver().to_vec(),
        signature.return_type().clone(),
    );
    resolved(
        context,
        &local,
        method.source(),
        Some(method.origin()),
        false,
        signatures,
    )
}

fn resolved(
    context: &ExpressionContext,
    signature: &FunctionSignature,
    source: &str,
    origin: Option<&FunctionOrigin>,
    visible: bool,
    signatures: &FunctionSignatures,
) -> Result<ResolvedBody, ExpressionError> {
    let parsed = parse(source).map_err(|error| decorate(error, signature.name(), origin))?;
    let parameters = parameter_symbols(signature.parameters());
    let typed = lower::function_with_signatures(
        &parsed,
        signature.return_type(),
        &|name| {
            parameters
                .get(name)
                .map(|(index, value_type)| SymbolTarget::Parameter(*index, value_type.clone()))
                .or_else(|| {
                    context
                        .build_input(name)
                        .map(|input| SymbolTarget::External(input.value_type().clone()))
                })
        },
        context,
        signatures,
    )
    .map_err(|error| decorate(error, signature.name(), origin))?;
    if typed.result_type() != signature.return_type() {
        return Err(decorate(
            ExpressionError::new(
                "EXPRESSION_RETURN_TYPE",
                format!(
                    "function `{}` declares return type {}, but its body has type {}",
                    signature.name(),
                    signature.return_type(),
                    typed.result_type()
                ),
                parsed.span,
            ),
            signature.name(),
            origin,
        ));
    }
    Ok(ResolvedBody::new(signature, source, origin, visible, typed))
}

fn parse(source: &str) -> Result<Expression, ExpressionError> {
    let expression =
        super::super::super::lexer::lex(source).and_then(super::super::super::parser::parse)?;
    matches!(&expression.kind, ExpressionKind::Block(_))
        .then_some(expression)
        .ok_or_else(|| {
            ExpressionError::new(
                "EXPRESSION_FUNCTION_BODY",
                "function body must be a block expression",
                0..source.len(),
            )
        })
}

fn parameter_symbols(parameters: &[FunctionParameter]) -> BTreeMap<&str, (usize, ValueType)> {
    parameters
        .iter()
        .enumerate()
        .map(|(index, value)| (value.name.as_str(), (index, value.value_type.clone())))
        .collect()
}

fn decorate(
    error: ExpressionError,
    name: &str,
    origin: Option<&FunctionOrigin>,
) -> ExpressionError {
    error.in_runtime_function(name, origin)
}

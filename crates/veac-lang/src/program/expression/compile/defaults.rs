use super::body::ResolvedBody;
use super::lower::{self, SymbolTarget};
use super::signature::{FunctionSignature, FunctionSignatures};
use crate::program::expression::ast::Expression;
use crate::program::expression::{Effect, ExpressionContext, ExpressionError, FunctionSummary};

pub(super) fn prepare(
    context: &ExpressionContext,
    owner: &FunctionSignature,
    signatures: &FunctionSignatures,
) -> Result<Vec<ResolvedBody>, ExpressionError> {
    owner
        .parameters()
        .iter()
        .filter_map(|parameter| parameter.default().map(|value| (parameter, value)))
        .map(|(parameter, value)| {
            let signature = FunctionSignature::new(
                value.thunk(),
                format!("{}::default::{}", owner.name(), parameter.name),
                Vec::new(),
                parameter.value_type.clone(),
            );
            let parsed =
                expression(value.source()).map_err(|error| decorate(error, owner, value))?;
            let typed = lower::function_with_signatures(
                &parsed,
                &parameter.value_type,
                &|name| {
                    context
                        .build_input(name)
                        .map(|input| SymbolTarget::External(input.value_type().clone()))
                },
                context,
                signatures,
            )
            .map_err(|error| decorate(error, owner, value))?;
            if typed.result_type() != &parameter.value_type {
                return Err(decorate(
                    ExpressionError::new(
                        "EXPRESSION_PARAMETER_DEFAULT_TYPE",
                        format!(
                            "parameter `{}` expects {}, but its default has type {}",
                            parameter.name,
                            parameter.value_type,
                            typed.result_type()
                        ),
                        parsed.span,
                    ),
                    owner,
                    value,
                ));
            }
            let body = ResolvedBody::new(&signature, value.source(), value.origin(), false, typed)
                .requiring_pure();
            Ok(body)
        })
        .collect()
}

pub(super) fn require_pure(
    summary: &FunctionSummary,
    body: &ResolvedBody,
) -> Result<(), ExpressionError> {
    if summary.effect() == Effect::Pure && !summary.contains_local_mutation() {
        return Ok(());
    }
    Err(body.decorate(ExpressionError::new(
        "EXPRESSION_PARAMETER_DEFAULT_EFFECT",
        "parameter default expressions must be pure",
        body.typed().root.span.clone(),
    )))
}

fn expression(source: &str) -> Result<Expression, ExpressionError> {
    super::super::lexer::lex(source).and_then(super::super::parser::parse)
}

fn decorate(
    error: ExpressionError,
    owner: &FunctionSignature,
    value: &crate::program::expression::FunctionDefault,
) -> ExpressionError {
    error.in_runtime_function(owner.name(), value.origin())
}

use super::super::Parser;
use crate::program::expression::lexer::TokenKind;
use crate::program::expression::{ExpressionError, FunctionEffect};

pub(super) fn parse(parser: &mut Parser) -> Result<(FunctionEffect, usize), ExpressionError> {
    let keyword = parser.advance();
    if !matches!(
        &keyword.kind,
        TokenKind::Symbol(value)
            if crate::vocabulary::control_uses::expression::FUNCTION_EFFECT.matches(value)
    ) {
        return Err(parser.type_error("expected `effect`", keyword.span));
    }
    let value = parser.advance();
    let TokenKind::Symbol(name) = &value.kind else {
        return Err(parser.type_error("expected a function effect", value.span));
    };
    let effect = FunctionEffect::parse(name).ok_or_else(|| {
        parser.type_error(
            "function effect must be `pure`, `local`, `emit`, or `any`",
            value.span.clone(),
        )
    })?;
    Ok((effect, value.span.end))
}

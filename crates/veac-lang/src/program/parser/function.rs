use std::collections::BTreeSet;

use super::{kind, Parser};
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::MAX_FUNCTION_PARAMETERS;
use crate::program::model::{FunctionBodyBinding, FunctionDecl, FunctionParameterDecl};
use crate::program::token::TokenKind;
use crate::vocabulary::control_uses::static_program as controls;
use crate::vocabulary::{accepts_expression_name, ExpressionNameKind};

pub(super) fn parse(parser: &mut Parser<'_>, exported: bool) -> Result<FunctionDecl, Diagnostic> {
    let start = parser.expect_control(controls::FUNCTION_DECLARATION)?;
    let (name, name_span) = parser.identifier("function name")?;
    if !accepts_expression_name(&name, ExpressionNameKind::Function) {
        return Err(parser.error(
            "PROGRAM_FUNCTION_NAME",
            format!("`{name}` is not a valid function name"),
            name_span,
        ));
    }
    parser.expect(TokenKind::LeftParen, "`(`")?;
    let parameters = parameters(parser)?;
    parser.expect(TokenKind::RightParen, "`)`")?;
    parser.expect(TokenKind::Arrow, "`->`")?;
    let return_type_syntax = kind::value(parser)?;
    let block = parser.raw_block()?;
    let body_span = block.span;
    let source = parser.source[body_span.start..body_span.end].to_owned();
    Ok(FunctionDecl {
        name,
        parameters,
        return_type_syntax,
        body: FunctionBodyBinding {
            source,
            span: body_span,
        },
        exported,
        span: start.join(block.span),
    })
}

pub(super) fn parameters(
    parser: &mut Parser<'_>,
) -> Result<Vec<FunctionParameterDecl>, Diagnostic> {
    parameters_with_limit(parser, MAX_FUNCTION_PARAMETERS)
}

pub(super) fn parameters_with_limit(
    parser: &mut Parser<'_>,
    limit: usize,
) -> Result<Vec<FunctionParameterDecl>, Diagnostic> {
    let mut parameters = Vec::new();
    let mut names = BTreeSet::new();
    while !parser.at(&TokenKind::RightParen) {
        let (name, name_span) = parser.identifier("function parameter name")?;
        if !accepts_expression_name(&name, ExpressionNameKind::Parameter) {
            return Err(parser.error(
                "PROGRAM_FUNCTION_PARAMETER_NAME",
                format!("`{name}` is not a valid function parameter name"),
                name_span,
            ));
        }
        if parameters.len() >= limit {
            return Err(parser.error(
                "PROGRAM_FUNCTION_PARAMETER_LIMIT",
                format!("function exceeds the {MAX_FUNCTION_PARAMETERS} total parameter limit"),
                name_span,
            ));
        }
        if !names.insert(name.clone()) {
            return Err(parser.error(
                "PROGRAM_DUPLICATE_PARAMETER",
                format!("function parameter `{name}` is declared more than once"),
                name_span,
            ));
        }
        parser.expect(TokenKind::Colon, "`:`")?;
        let (type_syntax, type_span) = kind::value_with_span(parser)?;
        parameters.push(FunctionParameterDecl {
            name,
            type_syntax,
            span: name_span.join(type_span),
        });
        if !parser.at(&TokenKind::Comma) {
            break;
        }
        parser.advance();
    }
    Ok(parameters)
}

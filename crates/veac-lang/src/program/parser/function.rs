use std::collections::BTreeSet;

use super::{kind, Parser};
use crate::program::diagnostic::Diagnostic;
use crate::program::expression::MAX_FUNCTION_PARAMETERS;
use crate::program::model::{
    FunctionBodyBinding, FunctionDecl, FunctionParameterDecl, ParameterDefaultBinding,
};
use crate::program::token::TokenKind;
use crate::vocabulary::control_uses::static_program as controls;
use crate::vocabulary::{accepts_expression_name, ExpressionNameKind};

pub(super) fn parse(parser: &mut Parser<'_>, exported: bool) -> Result<FunctionDecl, Diagnostic> {
    let syntax_start = parser.mark();
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
    let span = start.join(block.span);
    Ok(FunctionDecl {
        name,
        parameters,
        return_type_syntax,
        body: FunctionBodyBinding {
            span: body_span,
            syntax: block.syntax,
        },
        exported,
        span,
        syntax: parser.slice_from(syntax_start, span),
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
    let mut found_default = false;
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
        let default = if parser.at(&TokenKind::Equals) {
            parser.advance();
            let (syntax, span) = parser.expression_until_parameter_end()?;
            found_default = true;
            Some(ParameterDefaultBinding { span, syntax })
        } else {
            if found_default {
                return Err(parser.error(
                    "PROGRAM_REQUIRED_PARAMETER_AFTER_DEFAULT",
                    "required parameters cannot follow a parameter with a default",
                    name_span,
                ));
            }
            None
        };
        let span = default.as_ref().map_or(name_span.join(type_span), |value| {
            name_span.join(value.span)
        });
        parameters.push(FunctionParameterDecl {
            name,
            type_syntax,
            span,
            default,
        });
        if !parser.at(&TokenKind::Comma) {
            break;
        }
        parser.advance();
    }
    Ok(parameters)
}

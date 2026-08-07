use crate::program::diagnostic::Diagnostic;
use crate::program::expression::MAX_FUNCTION_PARAMETERS;
use crate::program::model::{FunctionBodyBinding, ImplDecl, MethodDecl};
use crate::program::token::TokenKind;
use crate::vocabulary::control_uses::static_program as controls;
use crate::vocabulary::{accepts_expression_name, ExpressionNameKind};

use super::{function, kind, Parser};

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<ImplDecl, Diagnostic> {
    let start = parser.expect_control(controls::IMPL_DECLARATION)?;
    let target = kind::value(parser)?;
    let (identity, identity_span) = parser.local_id("implementation identity")?;
    parser.expect(TokenKind::LeftBrace, "`{`")?;
    let mut methods = Vec::new();
    while !parser.at(&TokenKind::RightBrace) {
        if methods.len() >= crate::program::MAX_METHODS_PER_TYPE {
            return Err(parser.error(
                "PROGRAM_METHOD_LIMIT",
                format!(
                    "implementation exceeds the {} method limit",
                    crate::program::MAX_METHODS_PER_TYPE
                ),
                parser.current().span,
            ));
        }
        let exported = if parser.at_control(controls::EXPORT_MODIFIER) {
            parser.advance();
            true
        } else {
            false
        };
        if !parser.at_control(controls::FUNCTION_DECLARATION) {
            return Err(parser.error(
                "PROGRAM_IMPL_MEMBER",
                "implementation blocks contain only methods",
                parser.current().span,
            ));
        }
        methods.push(method(parser, exported)?);
    }
    let end = parser.expect(TokenKind::RightBrace, "`}`")?;
    Ok(ImplDecl {
        target,
        identity,
        identity_span,
        methods,
        span: start.join(end),
    })
}

fn method(parser: &mut Parser<'_>, exported: bool) -> Result<MethodDecl, Diagnostic> {
    let start = parser.expect_control(controls::FUNCTION_DECLARATION)?;
    let (name, name_span) = parser.identifier("method name")?;
    if !accepts_expression_name(&name, ExpressionNameKind::Function) {
        return Err(parser.error(
            "PROGRAM_METHOD_NAME",
            format!("`{name}` is not a valid method name"),
            name_span,
        ));
    }
    parser.expect(TokenKind::LeftParen, "`(`")?;
    parser.expect_control(controls::SELF_RECEIVER)?;
    let parameters = if parser.at(&TokenKind::Comma) {
        parser.advance();
        function::parameters_with_limit(parser, MAX_FUNCTION_PARAMETERS - 1)?
    } else {
        Vec::new()
    };
    parser.expect(TokenKind::RightParen, "`)`")?;
    parser.expect(TokenKind::Arrow, "`->`")?;
    let return_type_syntax = kind::value(parser)?;
    let block = parser.raw_block()?;
    let body_span = block.span;
    let source = parser.source[body_span.start..body_span.end].to_owned();
    Ok(MethodDecl {
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

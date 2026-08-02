use std::collections::BTreeMap;

use super::Parser;
use crate::program::diagnostic::Diagnostic;
use crate::program::model::{ExpressionBinding, InstanceDecl};
use crate::program::token::TokenKind;

pub(super) fn top_level(parser: &mut Parser<'_>) -> Result<InstanceDecl, Diagnostic> {
    parse(parser, false)
}

pub(super) fn local(parser: &mut Parser<'_>) -> Result<InstanceDecl, Diagnostic> {
    parse(parser, true)
}

fn parse(parser: &mut Parser<'_>, local: bool) -> Result<InstanceDecl, Diagnostic> {
    let start = parser.expect_word("instance")?;
    parser.expect_word("sequence")?;
    let (id, _) = if local {
        parser.local_id("local instance ID")?
    } else {
        parser.identifier("instance ID")?
    };
    parser.expect_word("from")?;
    let (component, _) = parser.qualified_name("component reference")?;
    parser.expect(TokenKind::LeftBrace, "`{`")?;
    let mut bindings = BTreeMap::new();
    let mut fills = BTreeMap::new();
    while !parser.at(&TokenKind::RightBrace) && !parser.at_eof() {
        if parser.at_word("bind") {
            parser.advance();
            let (name, span) = parser.identifier("parameter name")?;
            let (source, expression_span) = parser.expression_until_semicolon()?;
            let binding = ExpressionBinding {
                source,
                span: expression_span,
            };
            if bindings.insert(name, binding).is_some() {
                return Err(parser.error(
                    "PROGRAM_DUPLICATE_BINDING",
                    "parameter is bound more than once",
                    span,
                ));
            }
        } else if parser.at_word("fill") {
            parser.advance();
            let (name, span) = parser.identifier("slot name")?;
            let body = parser.raw_block()?;
            if fills.insert(name, body).is_some() {
                return Err(parser.error(
                    "PROGRAM_DUPLICATE_FILL",
                    "slot is filled more than once",
                    span,
                ));
            }
        } else {
            return Err(parser.error(
                "PROGRAM_INSTANCE_MEMBER",
                "instance members must be bind or fill",
                parser.current().span,
            ));
        }
    }
    let end = parser.expect(TokenKind::RightBrace, "`}`")?;
    Ok(InstanceDecl {
        component,
        id,
        bindings,
        fills,
        span: start.join(end),
    })
}

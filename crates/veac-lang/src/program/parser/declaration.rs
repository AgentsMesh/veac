use super::{kind, Parser};
use crate::program::diagnostic::Diagnostic;
use crate::program::model::{ConstDecl, ImportDecl, PresetDecl};
use crate::program::token::TokenKind;

pub(super) fn import(parser: &mut Parser<'_>) -> Result<ImportDecl, Diagnostic> {
    let start = parser.expect_word("import")?;
    let (path, _) = parser.string("relative module path")?;
    parser.expect_word("as")?;
    let (alias, _) = parser.identifier("import alias")?;
    let end = parser.expect(TokenKind::Semicolon, "`;`")?;
    Ok(ImportDecl {
        path,
        alias,
        span: start.join(end),
    })
}

pub(super) fn constant(parser: &mut Parser<'_>, exported: bool) -> Result<ConstDecl, Diagnostic> {
    let start = parser.expect_word("const")?;
    let value_type = kind::value(parser)?;
    let (name, _) = parser.identifier("constant name")?;
    parser.expect(TokenKind::Equals, "`=`")?;
    let (expression, expression_span) = parser.expression_until_semicolon()?;
    Ok(ConstDecl {
        name,
        value_type,
        expression,
        expression_span,
        exported,
        span: start.join(expression_span),
    })
}

pub(super) fn preset(parser: &mut Parser<'_>, exported: bool) -> Result<PresetDecl, Diagnostic> {
    let start = parser.expect_word("preset")?;
    let kind = kind::preset(parser)?;
    let (name, _) = parser.identifier("preset name")?;
    let body = parser.raw_block()?;
    Ok(PresetDecl {
        kind,
        name,
        exported,
        span: start.join(body.span),
        body,
    })
}

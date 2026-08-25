use super::{kind, Parser};
use crate::program::diagnostic::Diagnostic;
use crate::program::model::{ConstDecl, ImportDecl};
use crate::program::token::TokenKind;
use crate::vocabulary::control_uses::static_program as controls;

pub(super) fn import(parser: &mut Parser<'_>) -> Result<ImportDecl, Diagnostic> {
    let start = parser.expect_control(controls::IMPORT_DECLARATION)?;
    let (path, _) = parser.string("relative module path")?;
    parser.expect_control(controls::IMPORT_AS_CLAUSE)?;
    let (alias, _) = parser.identifier("import alias")?;
    let end = parser.expect(TokenKind::Semicolon, "`;`")?;
    Ok(ImportDecl {
        path,
        alias,
        span: start.join(end),
    })
}

pub(super) fn constant(parser: &mut Parser<'_>, exported: bool) -> Result<ConstDecl, Diagnostic> {
    let start = parser.expect_control(controls::CONST_DECLARATION)?;
    let type_syntax = kind::value(parser)?;
    let (name, _) = parser.identifier("constant name")?;
    parser.expect(TokenKind::Equals, "`=`")?;
    let (expression, end) = parser.expression_until_semicolon()?;
    let expression_span = expression.span;
    Ok(ConstDecl {
        name,
        type_syntax,
        expression,
        expression_span,
        exported,
        span: start.join(end),
    })
}

#[cfg(test)]
#[path = "declaration/tests.rs"]
mod tests;

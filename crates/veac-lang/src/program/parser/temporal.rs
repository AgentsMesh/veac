use super::Parser;
use crate::program::diagnostic::Diagnostic;
use crate::program::model::{FunctionBodyBinding, TemporalDecl, TemporalResourcePath};
use crate::program::token::TokenKind;
use crate::vocabulary::control_uses::static_program as controls;

mod property;
mod target;

pub(super) fn parse(parser: &mut Parser<'_>) -> Result<TemporalDecl, Diagnostic> {
    let syntax_start = parser.mark();
    let start = parser.expect_control(controls::TEMPORAL_DECLARATION)?;
    let property = property::parse(parser)?;
    parser.expect_control(controls::TEMPORAL_TARGET_CLAUSE)?;
    let target = target::parse(parser)?;
    let source = if parser.at_control(controls::TEMPORAL_SOURCE_CLAUSE) {
        parser.advance();
        parser.expect_control(controls::TEMPORAL_RESOURCE_KIND)?;
        Some(resource_path(parser)?)
    } else {
        None
    };
    let block = parser.raw_block()?;
    let span = start.join(block.span);
    Ok(TemporalDecl {
        property,
        target,
        source,
        body: FunctionBodyBinding {
            span: block.span,
            syntax: block.syntax,
        },
        span,
        syntax: parser.slice_from(syntax_start, span),
    })
}

fn resource_path(parser: &mut Parser<'_>) -> Result<TemporalResourcePath, Diagnostic> {
    parser.expect(TokenKind::LeftParen, "`(`")?;
    let project = segment(parser, "project key")?;
    let (resource, _) = parser.local_id("resource key")?;
    parser.expect(TokenKind::RightParen, "`)`")?;
    Ok(TemporalResourcePath { project, resource })
}

pub(super) fn segment(parser: &mut Parser<'_>, label: &str) -> Result<String, Diagnostic> {
    let (value, _) = parser.local_id(label)?;
    parser.expect(TokenKind::Comma, "`,`")?;
    Ok(value)
}

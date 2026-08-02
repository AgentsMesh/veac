use crate::authoring::lexer::{lex_with_limits, MAX_SOURCE_BYTES, MAX_TOKENS};
use crate::authoring::{Diagnostic, Diagnostics, Document};

use super::Parser;

pub fn parse(source: &str) -> Result<Document, Diagnostics> {
    parse_with_limits(source, MAX_SOURCE_BYTES, MAX_TOKENS)
}

pub(in crate::authoring) fn parse_with_limits(
    source: &str,
    source_limit: usize,
    token_limit: usize,
) -> Result<Document, Diagnostics> {
    parse_tokens(lex_with_limits(source, source_limit, token_limit))
}

fn parse_tokens(
    (tokens, diagnostics): (Vec<crate::authoring::lexer::Token>, Vec<Diagnostic>),
) -> Result<Document, Diagnostics> {
    let mut parser = Parser {
        tokens,
        cursor: 0,
        diagnostics,
    };
    let document = parser.project();
    if let Some(document) = &document {
        super::validate::document(&mut parser.diagnostics, document);
    }
    if parser.diagnostics.is_empty() {
        Ok(document.expect("a successful parse has a project"))
    } else {
        Err(parser.diagnostics.into())
    }
}

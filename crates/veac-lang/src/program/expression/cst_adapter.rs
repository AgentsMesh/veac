use super::ast::Expression;
use super::lexer::Token;
use super::ExpressionError;
use crate::program::syntax_document::{SyntaxDocument, SyntaxSlice};
use crate::program::token::Token as OuterToken;
use std::sync::Arc;

mod fragment;
mod mapping;
mod operator;
mod scan;

#[derive(Debug, Clone)]
pub(crate) struct ExpressionSyntax {
    source: Arc<str>,
    tokens: Arc<[OuterToken]>,
    slice: SyntaxSlice,
}

impl ExpressionSyntax {
    pub(crate) fn new(document: &SyntaxDocument, slice: &SyntaxSlice) -> Self {
        Self {
            source: document.source_arc(),
            tokens: document.tokens_arc(),
            slice: slice.clone(),
        }
    }

    pub(in crate::program::expression) fn parse(&self) -> Result<Expression, ExpressionError> {
        super::parser::parse(lex_parts(
            &self.source,
            &self.tokens[self.slice.tokens.clone()],
            &self.slice,
        )?)
    }
}

pub(in crate::program::expression) fn parse(
    source: &str,
    syntax: Option<&ExpressionSyntax>,
) -> Result<Expression, ExpressionError> {
    match syntax {
        Some(syntax) => syntax.parse(),
        None => super::parser::parse(super::lexer::lex(source)?),
    }
}

fn lex_parts(
    source: &str,
    outer: &[OuterToken],
    slice: &SyntaxSlice,
) -> Result<Vec<Token>, ExpressionError> {
    Adapter {
        source,
        outer,
        origin: slice.span.start,
        end: slice.span.end,
        cursor: 0,
        position: slice.span.start,
        output: Vec::new(),
    }
    .scan()
}

struct Adapter<'a> {
    source: &'a str,
    outer: &'a [OuterToken],
    origin: usize,
    end: usize,
    cursor: usize,
    position: usize,
    output: Vec<Token>,
}

#[cfg(test)]
#[path = "cst_adapter/operator_tests.rs"]
mod operator_tests;
#[cfg(test)]
#[path = "cst_adapter/tests.rs"]
mod tests;

use std::ops::Range;
use std::sync::Arc;

use crate::authoring::Span;

use super::token::Token;

#[derive(Debug, Clone)]
pub(crate) struct SyntaxDocument {
    source: Arc<str>,
    tokens: Arc<[Token]>,
    elements: Arc<[SyntaxElement]>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyntaxElement {
    pub kind: SyntaxElementKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum SyntaxElementKind {
    Token(usize),
    Trivia(TriviaKind),
    Gap,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TriviaKind {
    LineComment,
    BlockComment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SyntaxSlice {
    pub span: Span,
    pub tokens: Range<usize>,
}

impl SyntaxDocument {
    pub(crate) fn new(source: &str, tokens: Vec<Token>, elements: Vec<SyntaxElement>) -> Self {
        let value = Self {
            source: Arc::from(source),
            tokens: tokens.into(),
            elements: elements.into(),
        };
        debug_assert!(value.is_lossless());
        value
    }

    pub(crate) fn source(&self) -> &str {
        &self.source
    }

    pub(crate) fn source_arc(&self) -> Arc<str> {
        Arc::clone(&self.source)
    }

    pub(crate) fn tokens_arc(&self) -> Arc<[Token]> {
        Arc::clone(&self.tokens)
    }

    pub(crate) fn tokens(&self) -> &[Token] {
        &self.tokens
    }

    pub(crate) fn elements(&self) -> &[SyntaxElement] {
        &self.elements
    }

    pub(crate) fn text(&self, span: Span) -> &str {
        &self.source[span.start..span.end]
    }

    pub(crate) fn slice_text(&self, slice: &SyntaxSlice) -> &str {
        self.text(slice.span)
    }

    pub(crate) fn token_span(&self, tokens: Range<usize>) -> Option<Span> {
        if tokens.is_empty() {
            return None;
        }
        let first = self.tokens.get(tokens.start)?;
        let last = self.tokens.get(tokens.end.checked_sub(1)?)?;
        Some(first.span.join(last.span))
    }

    fn is_lossless(&self) -> bool {
        let mut end = 0;
        for element in self.elements.iter() {
            if element.span.start != end || element.span.end <= end {
                return false;
            }
            if let SyntaxElementKind::Token(index) = element.kind {
                if self.tokens.get(index).map(|token| token.span) != Some(element.span) {
                    return false;
                }
            }
            end = element.span.end;
        }
        end == self.source.len()
    }
}

#[cfg(test)]
#[path = "syntax_document/tests.rs"]
mod tests;

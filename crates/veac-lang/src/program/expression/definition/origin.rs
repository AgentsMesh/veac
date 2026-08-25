use std::fmt;
use std::ops::Range;

use crate::program::syntax_document::{SyntaxDocument, SyntaxSlice};

use super::super::cst_adapter::ExpressionSyntax;

#[derive(Clone)]
pub struct FunctionOrigin {
    source_id: String,
    body_span: Range<usize>,
    syntax: Option<ExpressionSyntax>,
}

impl FunctionOrigin {
    pub fn new(source_id: impl Into<String>, body_span: Range<usize>) -> Self {
        Self {
            source_id: source_id.into(),
            body_span,
            syntax: None,
        }
    }

    pub(crate) fn with_syntax(mut self, document: &SyntaxDocument, slice: &SyntaxSlice) -> Self {
        self.syntax = Some(ExpressionSyntax::new(document, slice));
        self
    }

    pub(crate) fn syntax(&self) -> Option<&ExpressionSyntax> {
        self.syntax.as_ref()
    }

    pub(crate) fn detached(&self) -> Self {
        Self::new(&self.source_id, self.body_span.clone())
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn body_span(&self) -> Range<usize> {
        self.body_span.clone()
    }

    pub fn absolute_span(&self, relative: Range<usize>) -> Range<usize> {
        let body_end = self.body_span.end.max(self.body_span.start);
        let start = self
            .body_span
            .start
            .saturating_add(relative.start)
            .min(body_end);
        let end = self
            .body_span
            .start
            .saturating_add(relative.end)
            .min(body_end)
            .max(start);
        start..end
    }
}

impl fmt::Debug for FunctionOrigin {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("FunctionOrigin")
            .field("source_id", &self.source_id)
            .field("body_span", &self.body_span)
            .finish()
    }
}

impl PartialEq for FunctionOrigin {
    fn eq(&self, other: &Self) -> bool {
        self.source_id == other.source_id && self.body_span == other.body_span
    }
}

impl Eq for FunctionOrigin {}

#[cfg(test)]
#[path = "origin/tests.rs"]
mod tests;

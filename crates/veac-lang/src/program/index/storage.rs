use crate::authoring::Span;
use crate::program::diagnostic::Diagnostic;
use crate::source_edit::{
    BodySite, DeclarationSite, ExpressionSite, SourceNodeRef, StatementSite, TextRange,
};

use super::{IndexedBody, IndexedDeclaration, IndexedExpression, IndexedStatement, SourceIndex};

impl SourceIndex {
    pub(super) fn insert(
        &mut self,
        path: &str,
        target: SourceNodeRef,
        site: ExpressionSite,
        source: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let value = IndexedExpression {
            source: source.to_owned(),
            range: range(span),
        };
        if self
            .expressions
            .insert((target.clone(), site), value)
            .is_some()
        {
            return Err(super::ambiguous(path, &target, span));
        }
        Ok(())
    }

    pub(super) fn insert_statement(
        &mut self,
        path: &str,
        target: SourceNodeRef,
        site: StatementSite,
        source: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let value = IndexedStatement {
            source: source.to_owned(),
            range: range(span),
        };
        if self
            .statements
            .insert((target.clone(), site), value)
            .is_some()
        {
            return Err(super::ambiguous(path, &target, span));
        }
        Ok(())
    }

    pub(super) fn insert_body(
        &mut self,
        path: &str,
        target: SourceNodeRef,
        site: BodySite,
        source: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let value = IndexedBody {
            source: source.to_owned(),
            range: range(span),
        };
        if self.bodies.insert((target.clone(), site), value).is_some() {
            return Err(super::ambiguous(path, &target, span));
        }
        Ok(())
    }

    pub(super) fn insert_declaration(
        &mut self,
        path: &str,
        target: SourceNodeRef,
        site: DeclarationSite,
        source: &str,
        span: Span,
    ) -> Result<(), Diagnostic> {
        let value = IndexedDeclaration {
            source: source[span.start..span.end].to_owned(),
            range: range(span),
        };
        if self
            .declarations
            .insert((target.clone(), site), value)
            .is_some()
        {
            return Err(super::ambiguous(path, &target, span));
        }
        Ok(())
    }
}

fn range(span: Span) -> TextRange {
    TextRange {
        start: span.start,
        end: span.end,
    }
}

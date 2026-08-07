use crate::authoring::Span;
use crate::program::expression::indexed_body_sites;
use crate::program::model::{FunctionBodyBinding, SurfaceFile};
use crate::source_edit::{ExpressionSite, SourceNodeRef, StatementSite};

use super::super::diagnostic::Diagnostic;
use super::SourceIndex;

pub(super) fn index(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    body: &FunctionBodyBinding,
) -> Result<(), Diagnostic> {
    let sites = indexed_body_sites(&body.source).map_err(|error| diagnostic(file, body, error))?;
    for expression in sites.expressions {
        let relative = expression.span;
        let absolute = Span {
            start: body.span.start + relative.start,
            end: body.span.start + relative.end,
        };
        index.insert(
            &file.path,
            target.clone(),
            ExpressionSite::BodyExpression {
                path: expression.path,
            },
            &body.source[relative],
            absolute,
        )?;
    }
    for statement in sites.statements {
        let relative = statement.span;
        let absolute = Span {
            start: body.span.start + relative.start,
            end: body.span.start + relative.end,
        };
        index.insert_statement(
            &file.path,
            target.clone(),
            StatementSite::BodyStatement {
                path: statement.path,
            },
            &body.source[relative],
            absolute,
        )?;
    }
    Ok(())
}

fn diagnostic(
    file: &SurfaceFile,
    body: &FunctionBodyBinding,
    error: crate::program::expression::ExpressionError,
) -> Diagnostic {
    let relative = error.span();
    Diagnostic::new(
        error.code(),
        &file.path,
        error.message(),
        Span {
            start: body.span.start + relative.start,
            end: body.span.start + relative.end,
        },
    )
}

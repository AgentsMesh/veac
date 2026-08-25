use crate::authoring::Span;
use crate::program::expression::indexed_temporal_attachments;
use crate::program::model::{FunctionBodyBinding, SurfaceFile};
use crate::source_edit::{BodySite, DeclarationSite, SourceNodeRef};

use super::super::diagnostic::Diagnostic;
use super::SourceIndex;

pub(super) fn index(
    index: &mut SourceIndex,
    file: &SurfaceFile,
    target: SourceNodeRef,
    body: &FunctionBodyBinding,
) -> Result<(), Diagnostic> {
    let body_source = file.syntax.slice_text(&body.syntax);
    let attachments =
        indexed_temporal_attachments(body_source).map_err(|error| diagnostic(file, body, error))?;
    for (ordinal, attachment) in attachments.into_iter().enumerate() {
        let ordinal = u32::try_from(ordinal).expect("expression node limit fits u32");
        let declaration = absolute(body.span, attachment.declaration);
        let animation_body = absolute(body.span, attachment.body);
        index.insert_declaration(
            &file.path,
            target.clone(),
            DeclarationSite::ComponentAnimation { ordinal },
            file.source(),
            declaration,
        )?;
        index.insert_body(
            &file.path,
            target.clone(),
            BodySite::ComponentAnimation {
                ordinal,
                property: super::temporal::property(attachment.property),
            },
            &file.source()[animation_body.start..animation_body.end],
            animation_body,
        )?;
    }
    Ok(())
}

fn absolute(base: Span, relative: std::ops::Range<usize>) -> Span {
    Span {
        start: base.start + relative.start,
        end: base.start + relative.end,
    }
}

fn diagnostic(
    file: &SurfaceFile,
    body: &FunctionBodyBinding,
    error: crate::program::expression::ExpressionError,
) -> Diagnostic {
    Diagnostic::new(
        error.code(),
        &file.path,
        error.message(),
        absolute(body.span, error.span()),
    )
}

#[cfg(test)]
#[path = "component_animation_tests.rs"]
mod tests;

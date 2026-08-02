use crate::authoring::{Diagnostic, LayerKind, SourceDecl, Span, TypedReference};
use std::collections::HashSet;

pub(super) fn validate<'a>(
    diagnostics: &mut Vec<Diagnostic>,
    source: &'a SourceDecl,
    layer_kind: LayerKind,
    resources: &HashSet<&'a str>,
    sequences: &HashSet<&'a str>,
) {
    match source {
        SourceDecl::Media { resource, .. } if !resources.contains(resource.id.value.as_str()) => {
            missing(diagnostics, resource)
        }
        SourceDecl::Sequence { sequence, .. }
            if !sequences.contains(sequence.id.value.as_str()) =>
        {
            missing(diagnostics, sequence)
        }
        _ => {}
    }
    let caption_source = matches!(source, SourceDecl::Caption { .. });
    if caption_source != (layer_kind == LayerKind::Caption) {
        push(
            diagnostics,
            "AUTHORING_SOURCE_LAYER",
            "caption layers require caption sources, which cannot be used on other layers",
            source_span(source),
        );
    }
    let text = match source {
        SourceDecl::Text { text, .. } => Some(&text.content),
        SourceDecl::Caption { caption, .. } => Some(&caption.text.content),
        _ => None,
    };
    if let Some(value) = text.filter(|value| value.value.is_empty()) {
        push(
            diagnostics,
            "AUTHORING_EMPTY_TEXT",
            "text and caption content cannot be empty",
            value.span,
        );
    }
}

fn missing(diagnostics: &mut Vec<Diagnostic>, value: &TypedReference) {
    push(
        diagnostics,
        "AUTHORING_REFERENCE_NOT_FOUND",
        "typed reference target does not exist",
        value.span,
    );
}

fn source_span(value: &SourceDecl) -> Span {
    match value {
        SourceDecl::Media { span, .. }
        | SourceDecl::Text { span, .. }
        | SourceDecl::Caption { span, .. }
        | SourceDecl::Generated { span, .. }
        | SourceDecl::Sequence { span, .. }
        | SourceDecl::Multicam { span, .. } => *span,
    }
}

fn push(diagnostics: &mut Vec<Diagnostic>, code: &'static str, message: &'static str, span: Span) {
    crate::authoring::diagnostic_budget::push(
        diagnostics,
        Diagnostic {
            code,
            message: message.to_owned(),
            span,
        },
    );
}

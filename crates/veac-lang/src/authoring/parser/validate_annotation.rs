use std::collections::HashSet;

use crate::authoring::{AnnotationDecl, AnnotationTargetDecl, AnnotationTimingDecl, Diagnostic};

use super::validate::Known;
use super::validate_numbers;

pub(super) fn validate(
    diagnostics: &mut Vec<Diagnostic>,
    values: &[AnnotationDecl],
    known: &Known<'_>,
) {
    let mut ids = HashSet::new();
    for value in values {
        if !ids.insert(value.id.value.as_str()) {
            diagnostics.push(Diagnostic::new(
                "AUTHORING_DUPLICATE_ID",
                format!("duplicate annotation id '{}'", value.id.value),
                value.id.span,
            ));
        }
        match &value.target {
            AnnotationTargetDecl::Project { .. } => {}
            AnnotationTargetDecl::Sequence(id) => {
                target(diagnostics, "sequence", id, &known.sequences)
            }
            AnnotationTargetDecl::Layer(id) => target(diagnostics, "layer", id, &known.layers),
            AnnotationTargetDecl::Item(id) => target(diagnostics, "item", id, &known.items),
            AnnotationTargetDecl::Resource(id) => {
                target(diagnostics, "resource", id, &known.resources)
            }
            AnnotationTargetDecl::Multicam(id) => diagnostics.push(Diagnostic::new(
                "AUTHORING_UNKNOWN_REFERENCE",
                format!("unknown multicam reference '{}'", id.value),
                id.span,
            )),
        }
        timing(diagnostics, &value.timing);
    }
}

fn target(
    diagnostics: &mut Vec<Diagnostic>,
    kind: &str,
    id: &crate::authoring::Identifier,
    known: &HashSet<&str>,
) {
    if !known.contains(id.value.as_str()) {
        diagnostics.push(Diagnostic::new(
            "AUTHORING_UNKNOWN_REFERENCE",
            format!("unknown {kind} reference '{}'", id.value),
            id.span,
        ));
    }
}

fn timing(diagnostics: &mut Vec<Diagnostic>, value: &AnnotationTimingDecl) {
    match value {
        AnnotationTimingDecl::Untimed { .. } => {}
        AnnotationTimingDecl::Point { at, .. } => {
            let _ = validate_numbers::time(diagnostics, "point", at, false);
        }
        AnnotationTimingDecl::Range { at, duration, .. } => {
            let _ = validate_numbers::time(diagnostics, "range start", at, false);
            let _ = validate_numbers::time(diagnostics, "range duration", duration, true);
        }
    }
}

use super::validate::Known;
use super::validate_numbers::record_span;
use crate::authoring::{
    ApplyDecl, ApplyItemTarget, ApplyScope, DeliveryDecl, Diagnostic, StructureDecl,
};
use std::collections::HashSet;

pub(super) fn structures(
    diagnostics: &mut Vec<Diagnostic>,
    structures: &[StructureDecl],
    known: &Known<'_>,
) {
    let mut ids = HashSet::new();
    for structure in structures {
        match structure {
            StructureDecl::Relation(value) => {
                structure_id(diagnostics, "relation", &value.id, &mut ids);
            }
            StructureDecl::Apply(value) => {
                structure_id(diagnostics, "apply", &value.id, &mut ids);
                apply(diagnostics, value, known);
            }
        }
    }
}

pub(super) fn deliveries(
    diagnostics: &mut Vec<Diagnostic>,
    deliveries: &[DeliveryDecl],
    known: &Known<'_>,
) {
    let mut ids = HashSet::new();
    for value in deliveries {
        if !ids.insert(&value.id.value) {
            crate::authoring::diagnostic_budget::push(
                diagnostics,
                Diagnostic {
                    code: "AUTHORING_DUPLICATE_ID",
                    message: "duplicate delivery id".to_owned(),
                    span: value.id.span,
                },
            );
        }
        delivery(diagnostics, value, known);
    }
}

fn apply(diagnostics: &mut Vec<Diagnostic>, value: &ApplyDecl, known: &Known<'_>) {
    match &value.scope {
        ApplyScope::CompositeBand { from, through, .. } => {
            known_id(diagnostics, "layer", from, &known.layers);
            known_id(diagnostics, "layer", through, &known.layers);
        }
        ApplyScope::Layer { layer, .. } => {
            known_id(diagnostics, "layer", layer, &known.layers);
        }
        ApplyScope::Items { targets, .. } => {
            for target in targets {
                if let ApplyItemTarget::Item(item) = target {
                    known_id(diagnostics, "item", item, &known.items);
                }
            }
        }
    }
    let mut stage_ids = HashSet::new();
    for stage in &value.pipeline {
        if !stage_ids.insert(stage.id().value.as_str()) {
            crate::authoring::diagnostic_budget::push(
                diagnostics,
                Diagnostic {
                    code: "AUTHORING_DUPLICATE_ID",
                    message: "duplicate apply stage id".to_owned(),
                    span: stage.id().span,
                },
            );
        }
    }
    record_span(diagnostics, &value.record);
}

fn delivery(diagnostics: &mut Vec<Diagnostic>, value: &DeliveryDecl, known: &Known<'_>) {
    if !known.sequences.contains(value.sequence.value.as_str()) {
        crate::authoring::diagnostic_budget::push(
            diagnostics,
            Diagnostic {
                code: "AUTHORING_REFERENCE_NOT_FOUND",
                message: format!("sequence `{}` does not exist", value.sequence.value),
                span: value.sequence.span,
            },
        );
    }
}

fn known_id(
    diagnostics: &mut Vec<Diagnostic>,
    kind: &str,
    value: &crate::authoring::Identifier,
    known: &HashSet<&str>,
) {
    if !known.contains(value.value.as_str()) {
        crate::authoring::diagnostic_budget::push(
            diagnostics,
            Diagnostic {
                code: "AUTHORING_REFERENCE_NOT_FOUND",
                message: format!("{kind} `{}` does not exist", value.value),
                span: value.span,
            },
        );
    }
}

fn structure_id<'a>(
    diagnostics: &mut Vec<Diagnostic>,
    kind: &'a str,
    value: &'a crate::authoring::Identifier,
    seen: &mut HashSet<(&'a str, &'a str)>,
) {
    if !seen.insert((kind, &value.value)) {
        crate::authoring::diagnostic_budget::push(
            diagnostics,
            Diagnostic {
                code: "AUTHORING_DUPLICATE_ID",
                message: format!("duplicate {kind} id"),
                span: value.span,
            },
        );
    }
}

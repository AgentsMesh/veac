use super::validate_numbers::{dimension, frame_rate, positive_integer, rational, record_span};
use crate::authoring::{Diagnostic, Document, ResourceIdentity, ResourceLocator};
use std::collections::HashSet;

pub(super) fn document(diagnostics: &mut Vec<Diagnostic>, document: &Document) {
    let project = &document.project;
    require_settings(diagnostics, document);
    duplicate_ids(
        diagnostics,
        "resource",
        project.resources.iter().map(|value| &value.id),
    );
    duplicate_ids(
        diagnostics,
        "sequence",
        project.sequences.iter().map(|value| &value.id),
    );
    duplicate_ids(
        diagnostics,
        "multicam",
        project.multicams.iter().map(|value| &value.id),
    );
    super::validate_multicam::declarations(diagnostics, project);
    super::validate_template_slot::validate(diagnostics, project);
    super::validate_output::validate(diagnostics, project);
    let resources: HashSet<_> = project
        .resources
        .iter()
        .map(|value| value.id.value.as_str())
        .collect();
    for resource in &project.resources {
        validate_locator(diagnostics, &resource.locator);
    }
    let sequences: HashSet<_> = project
        .sequences
        .iter()
        .map(|value| value.id.value.as_str())
        .collect();
    if !sequences.contains(project.entry.id.value.as_str()) {
        push(
            diagnostics,
            "AUTHORING_ENTRY_NOT_FOUND",
            "project entry sequence does not exist",
            project.entry.span,
        );
    }
    let mut layers = HashSet::new();
    let mut items = HashSet::new();
    for sequence in &project.sequences {
        for layer in &sequence.layers {
            insert_unique(diagnostics, "layer", &mut layers, &layer.id);
            for item in &layer.items {
                insert_unique(diagnostics, "item", &mut items, &item.id);
                super::validate_source::validate(
                    diagnostics,
                    &item.source,
                    layer.kind,
                    &resources,
                    &sequences,
                );
                super::validate_multicam::source(diagnostics, item, &project.multicams);
                record_span(diagnostics, &item.record);
                if let Some(value) = &item.mapping {
                    super::validate_mapping::mapping(diagnostics, value);
                }
            }
        }
    }
    let known = Known {
        resources,
        sequences,
        layers,
        items,
    };
    for sequence in &project.sequences {
        super::validate_refs::structures(diagnostics, &sequence.structures, &known);
    }
    super::validate_refs::outputs(diagnostics, &project.outputs, &known);
    super::validate_annotation::validate(diagnostics, &project.annotations, &known);
}

fn validate_locator(diagnostics: &mut Vec<Diagnostic>, locator: &ResourceLocator) {
    if let ResourceLocator::Remote {
        identity: ResourceIdentity::Sha256(value),
        ..
    } = locator
    {
        let valid =
            value.value.len() == 64 && value.value.bytes().all(|byte| byte.is_ascii_hexdigit());
        if !valid {
            push(
                diagnostics,
                "AUTHORING_RESOURCE_IDENTITY",
                "sha256 identity must contain exactly 64 hexadecimal digits",
                value.span,
            );
        }
    }
}

pub(super) struct Known<'a> {
    pub resources: HashSet<&'a str>,
    pub sequences: HashSet<&'a str>,
    pub layers: HashSet<&'a str>,
    pub items: HashSet<&'a str>,
}

fn require_settings(diagnostics: &mut Vec<Diagnostic>, document: &Document) {
    let settings = &document.project.settings;
    for (name, value) in [("sample-rate", settings.sample_rate.as_ref())] {
        if let Some(value) = value {
            positive_integer(diagnostics, name, value, "hz");
        }
    }
    if settings.timebase.is_none()
        || settings.canvas.is_none()
        || settings.frame_rate.is_none()
        || settings.sample_rate.is_none()
    {
        push(
            diagnostics,
            "AUTHORING_SETTINGS_REQUIRED",
            "settings require timebase, canvas, frame-rate, and sample-rate",
            settings.span,
        );
    }
    if let Some(value) = &settings.timebase {
        rational(diagnostics, "timebase", value);
    }
    if let Some((width, height)) = &settings.canvas {
        dimension(diagnostics, "canvas width", width);
        dimension(diagnostics, "canvas height", height);
    }
    if let Some(value) = &settings.frame_rate {
        frame_rate(diagnostics, value);
    }
}

fn duplicate_ids<'a>(
    diagnostics: &mut Vec<Diagnostic>,
    kind: &str,
    values: impl Iterator<Item = &'a crate::authoring::Identifier>,
) {
    let mut seen = HashSet::new();
    for value in values {
        insert_unique(diagnostics, kind, &mut seen, value);
    }
}

fn insert_unique<'a>(
    diagnostics: &mut Vec<Diagnostic>,
    kind: &str,
    seen: &mut HashSet<&'a str>,
    value: &'a crate::authoring::Identifier,
) {
    if !seen.insert(&value.value) {
        push(
            diagnostics,
            "AUTHORING_DUPLICATE_ID",
            &format!("duplicate {kind} id"),
            value.span,
        );
    }
}

fn push(
    diagnostics: &mut Vec<Diagnostic>,
    code: &'static str,
    message: &str,
    span: crate::authoring::Span,
) {
    diagnostics.push(Diagnostic {
        code,
        message: message.to_owned(),
        span,
    });
}

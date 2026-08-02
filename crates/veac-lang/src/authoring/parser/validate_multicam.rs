use crate::authoring::{
    Diagnostic, ItemDecl, MulticamDecl, NumberLiteral, ProjectDecl, ResourceKind, SourceDecl, Span,
};
use std::collections::{HashMap, HashSet};

pub(super) fn declarations(diagnostics: &mut Vec<Diagnostic>, project: &ProjectDecl) {
    let resources: HashMap<_, _> = project
        .resources
        .iter()
        .map(|value| (value.id.value.as_str(), value.kind))
        .collect();
    for group in &project.multicams {
        let mut angles = HashSet::new();
        if group.angles.len() < 2 {
            push(
                diagnostics,
                "AUTHORING_MULTICAM_ANGLE_COUNT",
                "multicam requires at least two angles",
                group.span,
            );
        }
        for angle in &group.angles {
            if !angles.insert(angle.id.value.as_str()) {
                push(
                    diagnostics,
                    "AUTHORING_DUPLICATE_ID",
                    "duplicate multicam angle id",
                    angle.id.span,
                );
            }
            match resources.get(angle.resource.id.value.as_str()) {
                Some(ResourceKind::Video) => {}
                Some(_) => push(
                    diagnostics,
                    "AUTHORING_MULTICAM_RESOURCE_TYPE",
                    "multicam angles require video resources",
                    angle.resource.span,
                ),
                None => push(
                    diagnostics,
                    "AUTHORING_REFERENCE_NOT_FOUND",
                    "multicam angle resource does not exist",
                    angle.resource.span,
                ),
            }
            super::validate_numbers::time(
                diagnostics,
                "multicam source-offset",
                &angle.source_offset,
                false,
            );
        }
        if !angles.contains(group.sync.reference.id.value.as_str()) {
            push(
                diagnostics,
                "AUTHORING_MULTICAM_REFERENCE",
                "multicam sync reference angle does not exist",
                group.sync.reference.span,
            );
        }
    }
}

pub(super) fn source(diagnostics: &mut Vec<Diagnostic>, item: &ItemDecl, groups: &[MulticamDecl]) {
    let SourceDecl::Multicam {
        group,
        switches,
        span,
    } = &item.source
    else {
        return;
    };
    let Some(declaration) = groups.iter().find(|value| value.id.value == group.id.value) else {
        push(
            diagnostics,
            "AUTHORING_REFERENCE_NOT_FOUND",
            "multicam group does not exist",
            group.span,
        );
        return;
    };
    let angles: HashSet<_> = declaration
        .angles
        .iter()
        .map(|value| value.id.value.as_str())
        .collect();
    let mut expected = 0.0;
    for value in switches {
        let at = super::validate_numbers::time(diagnostics, "multicam switch at", &value.at, false);
        let duration = super::validate_numbers::time(
            diagnostics,
            "multicam switch duration",
            &value.duration,
            true,
        );
        if at.is_some_and(|at| (at - expected).abs() > 1e-9) {
            push(
                diagnostics,
                "AUTHORING_MULTICAM_PARTITION",
                "multicam switches must form a contiguous clip-local partition",
                value.at.span,
            );
        }
        if !angles.contains(value.angle.id.value.as_str()) {
            push(
                diagnostics,
                "AUTHORING_MULTICAM_SWITCH_ANGLE",
                "multicam switch angle does not exist in its group",
                value.angle.span,
            );
        }
        if let (Some(at), Some(duration)) = (at, duration) {
            expected = at + duration;
        }
    }
    let item_duration = seconds(&item.record.duration);
    if switches.is_empty()
        || item_duration.is_some_and(|duration| (expected - duration).abs() > 1e-9)
    {
        push(
            diagnostics,
            "AUTHORING_MULTICAM_PARTITION",
            "multicam switches must cover the complete item duration",
            *span,
        );
    }
}

fn seconds(value: &NumberLiteral) -> Option<f64> {
    value
        .raw
        .strip_suffix("ms")
        .and_then(|raw| raw.parse::<f64>().ok())
        .map(|value| value / 1_000.0)
        .or_else(|| {
            value
                .raw
                .strip_suffix('s')
                .and_then(|raw| raw.parse::<f64>().ok())
        })
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

use std::collections::HashSet;

use crate::authoring::{
    Diagnostic, MappingDecl, MappingKey, NumberLiteral, SourceOutOfRangeDecl, Span,
};

use super::validate_numbers::{push, time};

pub(super) fn mapping(diagnostics: &mut Vec<Diagnostic>, value: &MappingDecl) {
    match value {
        MappingDecl::Linear {
            from, to, outside, ..
        } => {
            let from_value = source_time(diagnostics, "mapping from", from, *outside);
            let to_value = source_time(diagnostics, "mapping to", to, *outside);
            if from_value
                .zip(to_value)
                .is_some_and(|(from, to)| to == from)
            {
                push(
                    diagnostics,
                    "AUTHORING_MAPPING_RANGE",
                    "linear mapping to must differ from from".to_owned(),
                    to,
                );
            }
        }
        MappingDecl::Freeze { source, .. } => {
            source_time(
                diagnostics,
                "freeze source",
                source,
                SourceOutOfRangeDecl::Strict,
            );
        }
        MappingDecl::Curve {
            keys,
            outside,
            span,
        } => curve(diagnostics, keys, *outside, *span),
    }
}

fn curve(
    diagnostics: &mut Vec<Diagnostic>,
    keys: &[MappingKey],
    outside: SourceOutOfRangeDecl,
    span: Span,
) {
    let mut ids = HashSet::new();
    let mut previous = None;
    for key in keys {
        if !ids.insert(&key.id.value) {
            crate::authoring::diagnostic_budget::push(
                diagnostics,
                Diagnostic {
                    code: "AUTHORING_DUPLICATE_ID",
                    message: "duplicate mapping key id".to_owned(),
                    span: key.id.span,
                },
            );
        }
        let at = time(diagnostics, "curve key at", &key.at, false);
        source_time(diagnostics, "curve key source", &key.source, outside);
        if at
            .zip(previous)
            .is_some_and(|(at, previous)| at <= previous)
        {
            crate::authoring::diagnostic_budget::push(
                diagnostics,
                Diagnostic {
                    code: "AUTHORING_MAPPING_KEY_ORDER",
                    message: "curve key times must be strictly increasing".to_owned(),
                    span: key.at.span,
                },
            );
        }
        previous = at.or(previous);
    }
    if keys.is_empty() {
        crate::authoring::diagnostic_budget::push(
            diagnostics,
            Diagnostic {
                code: "AUTHORING_MAPPING_KEYS",
                message: "curve mapping requires at least one key".to_owned(),
                span,
            },
        );
    }
}

fn source_time(
    diagnostics: &mut Vec<Diagnostic>,
    name: &str,
    value: &NumberLiteral,
    outside: SourceOutOfRangeDecl,
) -> Option<f64> {
    if !matches!(
        outside,
        SourceOutOfRangeDecl::HoldFirst | SourceOutOfRangeDecl::HoldBoth
    ) {
        return time(diagnostics, name, value, false);
    }
    let raw = value.raw.as_str();
    let parsed = raw
        .strip_suffix("ms")
        .map(|value| (value, 0.001))
        .or_else(|| raw.strip_suffix('s').map(|value| (value, 1.0)))
        .and_then(|(value, scale)| value.parse::<f64>().ok().map(|value| value * scale));
    if parsed.is_none_or(|value| !value.is_finite()) {
        push(
            diagnostics,
            "AUTHORING_TIME_LITERAL",
            format!("{name} must be a finite source time using s or ms"),
            value,
        );
        None
    } else {
        parsed
    }
}

use crate::authoring::{Diagnostic, NumberLiteral, RecordSpan};

pub(super) fn positive_integer(
    diagnostics: &mut Vec<Diagnostic>,
    name: &str,
    value: &NumberLiteral,
    unit: &str,
) {
    let parsed = value
        .raw
        .strip_suffix(unit)
        .and_then(|raw| raw.parse::<u64>().ok());
    if parsed.is_none_or(|value| value == 0) {
        push(
            diagnostics,
            "AUTHORING_POSITIVE_INTEGER",
            format!("{name} must be a positive integer in {unit}"),
            value,
        );
    }
}

pub(super) fn rational(diagnostics: &mut Vec<Diagnostic>, name: &str, value: &NumberLiteral) {
    let parts: Vec<_> = value.raw.split('/').collect();
    let valid = matches!(
        parts.as_slice(),
        [numerator, denominator]
            if numerator.parse::<u64>().is_ok_and(|value| value > 0)
                && denominator.parse::<u64>().is_ok_and(|value| value > 0)
    );
    if !valid {
        push(
            diagnostics,
            "AUTHORING_RATIONAL_LITERAL",
            format!("{name} must be a positive rational such as 1/1000000"),
            value,
        );
    }
}

pub(super) fn dimension(diagnostics: &mut Vec<Diagnostic>, name: &str, value: &NumberLiteral) {
    let parsed = value
        .raw
        .strip_suffix("px")
        .and_then(|raw| raw.parse::<u64>().ok());
    if parsed.is_none_or(|value| value == 0) {
        push(
            diagnostics,
            "AUTHORING_DIMENSION_LITERAL",
            format!("{name} must be a positive px dimension"),
            value,
        );
    }
}

pub(super) fn frame_rate(diagnostics: &mut Vec<Diagnostic>, value: &NumberLiteral) {
    let Some(raw) = value.raw.strip_suffix("fps") else {
        push(
            diagnostics,
            "AUTHORING_FRAME_RATE_LITERAL",
            "frame-rate must use fps".to_owned(),
            value,
        );
        return;
    };
    let parts: Vec<_> = raw.split('/').collect();
    let valid = match parts.as_slice() {
        [numerator] => numerator.parse::<u64>().is_ok_and(|value| value > 0),
        [numerator, denominator] => {
            numerator.parse::<u64>().is_ok_and(|value| value > 0)
                && denominator.parse::<u64>().is_ok_and(|value| value > 0)
        }
        _ => false,
    };
    if !valid {
        push(
            diagnostics,
            "AUTHORING_FRAME_RATE_LITERAL",
            "frame-rate must be 30fps or a positive rational such as 30000/1001fps".to_owned(),
            value,
        );
    }
}

pub(super) fn record_span(diagnostics: &mut Vec<Diagnostic>, value: &RecordSpan) {
    time(diagnostics, "span at", &value.at, false);
    time(diagnostics, "span duration", &value.duration, true);
}

pub(super) fn time(
    diagnostics: &mut Vec<Diagnostic>,
    name: &str,
    value: &NumberLiteral,
    positive: bool,
) -> Option<f64> {
    let raw = value.raw.as_str();
    let numeric = raw
        .strip_suffix("ms")
        .map(|value| (value, 0.001))
        .or_else(|| raw.strip_suffix('s').map(|value| (value, 1.0)));
    let parsed = numeric.and_then(|(value, scale)| value.parse::<f64>().ok().map(|v| v * scale));
    let invalid = parsed
        .is_none_or(|parsed| !parsed.is_finite() || parsed < 0.0 || positive && parsed == 0.0);
    if invalid {
        push(
            diagnostics,
            "AUTHORING_TIME_LITERAL",
            format!(
                "{name} must be {}time using s or ms",
                if positive {
                    "positive "
                } else {
                    "non-negative "
                }
            ),
            value,
        );
        None
    } else {
        parsed
    }
}

pub(super) fn push(
    diagnostics: &mut Vec<Diagnostic>,
    code: &'static str,
    message: String,
    value: &NumberLiteral,
) {
    diagnostics.push(Diagnostic {
        code,
        message,
        span: value.span,
    });
}

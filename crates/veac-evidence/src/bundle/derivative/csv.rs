use crate::{AssertionKind, AssertionStatus, EvidenceReportV1};

pub(crate) fn render(report: &EvidenceReportV1) -> Vec<u8> {
    let mut output = String::from("assertion_id,kind,status,message,metric,value\n");
    for assertion in &report.assertions {
        if assertion.metrics.is_empty() {
            row(
                &mut output,
                &assertion.id,
                kind(assertion.kind),
                status(assertion.status),
                &assertion.message,
                "",
                "",
            );
        }
        for (metric, value) in &assertion.metrics {
            row(
                &mut output,
                &assertion.id,
                kind(assertion.kind),
                status(assertion.status),
                &assertion.message,
                metric,
                &value.to_string(),
            );
        }
    }
    output.into_bytes()
}

fn row(
    output: &mut String,
    id: &str,
    kind: &str,
    status: &str,
    message: &str,
    key: &str,
    value: &str,
) {
    for (index, field) in [id, kind, status, message, key, value]
        .into_iter()
        .enumerate()
    {
        if index > 0 {
            output.push(',');
        }
        field_value(output, field);
    }
    output.push('\n');
}

fn field_value(output: &mut String, value: &str) {
    if value.contains([',', '"', '\n', '\r']) {
        output.push('"');
        output.push_str(&value.replace('"', "\"\""));
        output.push('"');
    } else {
        output.push_str(value);
    }
}

fn kind(value: AssertionKind) -> &'static str {
    match value {
        AssertionKind::DecodeComplete => "decode_complete",
        AssertionKind::Alpha => "alpha",
        AssertionKind::PixelDiff => "pixel_diff",
        AssertionKind::Bounds => "bounds",
        AssertionKind::LayerOrder => "layer_order",
        AssertionKind::CompositeOver => "composite_over",
        AssertionKind::RevealOrder => "reveal_order",
        AssertionKind::MotionProfile => "motion_profile",
    }
}

fn status(value: AssertionStatus) -> &'static str {
    match value {
        AssertionStatus::Pass => "pass",
        AssertionStatus::Fail => "fail",
        AssertionStatus::Error => "error",
    }
}

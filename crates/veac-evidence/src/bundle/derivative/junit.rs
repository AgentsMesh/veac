use crate::{AssertionStatus, EvidenceReportV1};

pub(crate) fn render(report: &EvidenceReportV1) -> Vec<u8> {
    let failures = report
        .assertions
        .iter()
        .filter(|value| value.status == AssertionStatus::Fail)
        .count();
    let errors = report
        .assertions
        .iter()
        .filter(|value| value.status == AssertionStatus::Error)
        .count();
    let mut output = format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?><testsuite name=\"{}\" tests=\"{}\" failures=\"{}\" errors=\"{}\">",
        escaped(&report.suite_id),
        report.assertions.len(),
        failures,
        errors
    );
    for assertion in &report.assertions {
        output.push_str("<testcase name=\"");
        output.push_str(&escaped(&assertion.id));
        output.push_str("\">");
        match assertion.status {
            AssertionStatus::Pass => {}
            AssertionStatus::Fail => element(&mut output, "failure", &assertion.message),
            AssertionStatus::Error => element(&mut output, "error", &assertion.message),
        }
        output.push_str("</testcase>");
    }
    output.push_str("</testsuite>");
    output.into_bytes()
}

fn element(output: &mut String, name: &str, message: &str) {
    output.push('<');
    output.push_str(name);
    output.push_str(" message=\"");
    output.push_str(&escaped(message));
    output.push_str("\"/>");
}

fn escaped(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
#[path = "junit/coverage_tests.rs"]
mod coverage_tests;

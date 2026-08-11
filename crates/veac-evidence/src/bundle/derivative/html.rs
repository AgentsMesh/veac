use crate::{AssertionStatus, EvidenceReportV1};

pub(crate) fn render(report: &EvidenceReportV1) -> Vec<u8> {
    let mut output = String::from(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><title>VEAC evidence</title>",
    );
    output.push_str("<style>body{font:14px system-ui;margin:24px;color:#171717}table{border-collapse:collapse;width:100%}th,td{border:1px solid #ccc;padding:8px;text-align:left}.pass{color:#147a36}.fail,.error{color:#b42318}</style>");
    output.push_str("<h1>Evidence results</h1><table><thead><tr><th>Assertion</th><th>Status</th><th>Message</th></tr></thead><tbody>");
    for assertion in &report.assertions {
        let status = match assertion.status {
            AssertionStatus::Pass => "pass",
            AssertionStatus::Fail => "fail",
            AssertionStatus::Error => "error",
        };
        output.push_str("<tr><td>");
        escape(&mut output, &assertion.id);
        output.push_str("</td><td class=\"");
        output.push_str(status);
        output.push_str("\">");
        output.push_str(status);
        output.push_str("</td><td>");
        escape(&mut output, &assertion.message);
        output.push_str("</td></tr>");
    }
    output.push_str("</tbody></table></html>");
    output.into_bytes()
}

fn escape(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '&' => output.push_str("&amp;"),
            '<' => output.push_str("&lt;"),
            '>' => output.push_str("&gt;"),
            '"' => output.push_str("&quot;"),
            '\'' => output.push_str("&#39;"),
            value => output.push(value),
        }
    }
}

use std::collections::BTreeMap;

use super::render;
use crate::{
    AssertionKind, AssertionResult, AssertionStatus, EvidenceOutcome, EvidenceReportV1,
    EVIDENCE_REPORT_SCHEMA_VERSION,
};

#[test]
fn junit_distinguishes_failures_and_errors() {
    let result = |id: &str, status: AssertionStatus| AssertionResult {
        id: id.to_owned(),
        kind: AssertionKind::DecodeComplete,
        status,
        message: "bad <value>".into(),
        metrics: BTreeMap::new(),
        series: BTreeMap::new(),
    };
    let report = EvidenceReportV1 {
        schema_version: EVIDENCE_REPORT_SCHEMA_VERSION,
        suite_id: "suite & contract".into(),
        outcome: EvidenceOutcome::Error,
        assertions: vec![
            result("failed", AssertionStatus::Fail),
            result("error", AssertionStatus::Error),
        ],
    };
    let xml = String::from_utf8(render(&report)).unwrap();
    assert!(xml.contains("failures=\"1\" errors=\"1\""));
    assert!(xml.contains("<failure message=\"bad &lt;value&gt;\"/>"));
    assert!(xml.contains("<error message=\"bad &lt;value&gt;\"/>"));
}

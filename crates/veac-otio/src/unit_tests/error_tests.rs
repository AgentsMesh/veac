use std::error::Error;

use crate::*;

#[test]
fn error_variants_have_stable_context_and_sources() {
    let json: OtioError = serde_json::from_str::<serde_json::Value>("{")
        .unwrap_err()
        .into();
    assert!(json.to_string().starts_with("OTIO JSON failed:"));
    assert!(json.source().is_some());

    let contract = OtioError::contract("bad contract");
    assert_eq!(contract.to_string(), "OTIO contract failed: bad contract");
    assert!(contract.source().is_none());
    assert_eq!(
        OtioError::time("bad time").to_string(),
        "OTIO time failed: bad time"
    );
    assert_eq!(
        OtioError::Edit("bad edit".to_owned()).to_string(),
        "OTIO edit proposal failed: bad edit"
    );
    let mut report = OtioLossReport::default();
    report.push("/x", "field", "reason", false);
    assert_eq!(
        OtioError::Loss(report).to_string(),
        "OTIO conversion has 1 loss(es)"
    );
}

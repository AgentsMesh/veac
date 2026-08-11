use std::error::Error;

use super::*;

#[test]
fn public_errors_expose_stable_context_and_sources() {
    let decode = EvidenceDecodeError::new("suite.id", "invalid");
    assert_eq!(decode.to_string(), "suite.id: invalid");
    let cases = [
        EvidenceAuthoringError::Load("missing".into()),
        EvidenceAuthoringError::Decode(decode),
        EvidenceAuthoringError::Validation(crate::ValidationError { issues: vec![] }),
        EvidenceAuthoringError::Identity(crate::BundleError::Encode("bad".into())),
    ];
    for error in cases {
        assert!(!error.to_string().is_empty());
        assert!(error.source().is_none());
    }
    let language = crate::build_evidence_source("not evidence").unwrap_err();
    assert!(language.to_string().contains("evidence language failed"));
}

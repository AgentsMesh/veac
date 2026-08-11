use super::{build_error, evidence_result::EvidenceSummary, gate_evidence};
use veac_build::{BuildError, BuildErrorKind};

#[test]
fn evidence_test_gate_requires_a_nonempty_pass() {
    assert!(gate_evidence(EvidenceSummary {
        total: 1,
        passed: 1,
        failed: 0,
        errors: 0,
    })
    .is_ok());

    let error = gate_evidence(EvidenceSummary {
        total: 2,
        passed: 0,
        failed: 1,
        errors: 1,
    })
    .unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "PROJECT_TEST_FAILED");
    assert!(error.to_string().contains("1 failed, 1 errors"));

    assert!(gate_evidence(EvidenceSummary::default()).is_err());
}

#[test]
fn build_errors_keep_kind_specific_cli_codes() {
    let cases = [
        (BuildErrorKind::InvalidContract, "PROJECT_BUILD_CONTRACT"),
        (BuildErrorKind::ResourceLimit, "PROJECT_BUILD_LIMIT"),
        (BuildErrorKind::Cache, "PROJECT_BUILD_CACHE"),
        (BuildErrorKind::Cancelled, "PROJECT_BUILD_CANCELLED"),
        (BuildErrorKind::Internal, "PROJECT_BUILD_INTERNAL"),
    ];
    for (kind, code) in cases {
        let error = build_error(BuildError::new(kind, "failure"));
        assert_eq!(error.diagnostics()[0].code, code);
        assert_eq!(
            error.is_resource_limit(),
            kind == BuildErrorKind::ResourceLimit
        );
    }
}

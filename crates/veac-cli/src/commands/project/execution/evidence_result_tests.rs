use super::contract;

#[test]
fn evidence_contract_error_uses_the_stable_machine_code() {
    let error = contract("missing evidence output".to_owned());
    assert_eq!(error.diagnostics()[0].code, "PROJECT_EVIDENCE_CONTRACT");
    assert_eq!(error.diagnostics()[0].message, "missing evidence output");
}

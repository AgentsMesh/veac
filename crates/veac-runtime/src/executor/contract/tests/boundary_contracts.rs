use super::super::{active, validate_while};
use super::support::bundle;
use crate::RuntimeErrorKind;
use veac_codegen::emitter::BackendOutput;

#[test]
fn empty_bundles_and_expired_guards_fail_before_external_work() {
    let temp = tempfile::tempdir().unwrap();
    let mut value = bundle(&temp.path().join("output.vtt"));
    value.tasks.clear();
    let error = match validate_while(&value, || true) {
        Ok(_) => panic!("empty bundle must be rejected"),
        Err(error) => error,
    };
    assert!(error.message.contains("at least one task"));

    let error = active(&mut || false).unwrap_err();
    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
    assert!(error.message.contains("setup deadline"));
}

#[test]
fn multi_file_declarations_are_collected_before_action_validation() {
    let temp = tempfile::tempdir().unwrap();
    let mut value = bundle(&temp.path().join("primary.vtt"));
    value.tasks[0].output = BackendOutput::Files {
        paths: vec![
            temp.path().join("primary.vtt"),
            temp.path().join("secondary.vtt"),
        ],
    };
    let error = match validate_while(&value, || true) {
        Ok(_) => panic!("write action with multiple files must be rejected"),
        Err(error) => error,
    };
    assert!(error.message.contains("write-file task"));
}

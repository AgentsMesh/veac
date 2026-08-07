use std::fs;

use super::MAIN;

#[test]
fn path_build_returns_the_canonical_root_and_executes_the_program() {
    let temp = tempfile::tempdir().unwrap();
    let entry = temp.path().join("main.veac");
    fs::write(&entry, MAIN).unwrap();

    let (root, built) = super::super::build_path_with_root(&entry).unwrap();
    assert_eq!(root, temp.path().canonicalize().unwrap());
    assert_eq!(built.root_module(), "main.veac");
}

#[test]
fn path_preparation_wraps_loader_failures_in_a_program_diagnostic() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing.veac");
    let diagnostics = super::super::prepare_path_with_root(&missing).unwrap_err();
    assert_eq!(diagnostics.as_slice()[0].code, "PROGRAM_ENTRY_LOAD");
    assert_eq!(
        diagnostics.as_slice()[0].path,
        missing.display().to_string()
    );
}

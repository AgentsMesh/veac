use super::SourceGraphLock;

#[test]
fn revalidation_rejects_a_removed_lock_path() {
    let temp = tempfile::tempdir().unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    std::fs::remove_file(temp.path().join(".veac-source.lock")).unwrap();

    let error = lock.revalidate(temp.path()).unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert!(error
        .to_string()
        .contains("source lock path is unavailable"));
}

#[test]
fn commit_rejects_an_unconfined_module_before_opening_it() {
    let temp = tempfile::tempdir().unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();

    let error = lock
        .commit_module(temp.path(), "../outside.veac", "", "changed")
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_EDIT_MODULE");
    assert!(!temp.path().parent().unwrap().join("outside.veac").exists());
}

#[test]
fn same_length_stale_source_is_rejected_by_byte_comparison() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "actual").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();

    let error = lock
        .commit_module(temp.path(), "main.veac", "stale!", "after")
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(std::fs::read_to_string(source).unwrap(), "actual");
}

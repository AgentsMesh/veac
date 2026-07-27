use std::path::Path;

use super::super::store;
use super::support::bundle;

#[test]
fn future_store_paths_normalize_and_reject_output_overlap() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output.srt");
    let value = bundle(&output);
    store::validate(&value, &temp.path().join("future/store")).unwrap();
    assert!(store::validate(&value, &output)
        .unwrap_err()
        .message
        .contains("artifact store management directory"));
    assert!(store::validate(&value, &output.join("nested"))
        .unwrap_err()
        .message
        .contains("artifact store management directory"));
}

#[test]
fn store_paths_reject_parent_traversal_and_missing_ancestors() {
    let temp = tempfile::tempdir().unwrap();
    let value = bundle(&temp.path().join("output.srt"));
    assert!(store::validate(&value, Path::new("future/../store"))
        .unwrap_err()
        .message
        .contains("may not contain '..'"));
    assert!(store::validate(&value, Path::new(""))
        .unwrap_err()
        .message
        .contains("no existing ancestor"));
}

#[cfg(unix)]
#[test]
fn dangling_store_symlink_reports_canonicalization_failure() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("dangling");
    symlink(temp.path().join("missing"), &root).unwrap();
    let error = store::validate(&bundle(&temp.path().join("output.srt")), &root).unwrap_err();
    assert!(error
        .message
        .contains("cannot validate artifact store path"));
}

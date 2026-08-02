use std::fs::File;

use rustix::fs::fstat;

use super::*;
use crate::executor::staging::directory::Directory;

#[test]
fn rename_failure_before_install_never_crosses_the_commit_point() {
    let temp = tempfile::tempdir().unwrap();
    let root = Directory::open(temp.path()).unwrap();
    let source = root.create_child("source").unwrap();
    let target = root.create_child("target").unwrap();
    let expected = source.write_all_sync("payload", b"content").unwrap();
    let path = temp.path().join("source/payload");

    let failure = source
        .rename_bound_to_with("payload", expected, &target, "output", || {
            std::fs::remove_file(path).unwrap();
        })
        .unwrap_err();

    assert!(!failure.crossed_commit);
    assert!(failure.error.message.contains("rename transaction path"));
    assert!(target.missing("output").unwrap());
}

#[test]
fn numeric_identity_conversions_reject_negative_kernel_values() {
    assert!(identity_number(-1_i64, "fixture")
        .unwrap_err()
        .message
        .contains("out of range"));
    assert!(nonnegative_size(-1_i64)
        .unwrap_err()
        .message
        .contains("file size is negative"));

    let temp = tempfile::tempdir().unwrap();
    let file = File::create(temp.path().join("payload")).unwrap();
    let mut metadata = fstat(&file).unwrap();
    metadata.st_size = -1;
    assert!(from_stat(metadata)
        .unwrap_err()
        .message
        .contains("file size is negative"));
}

#[test]
fn opened_identity_rejects_multiply_linked_files() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("payload");
    std::fs::write(&path, b"content").unwrap();
    std::fs::hard_link(&path, temp.path().join("alias")).unwrap();

    let error = opened_identity(&File::open(path).unwrap()).unwrap_err();
    assert!(error.message.contains("exclusively linked regular file"));
}

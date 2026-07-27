use super::super::directory::{Directory, EntryIdentity};

#[test]
fn identity_bound_operations_reject_a_missing_output() {
    let temp = tempfile::tempdir().unwrap();
    let directory = Directory::open(temp.path()).unwrap();

    let error = directory
        .require(
            "missing.out",
            EntryIdentity {
                device: 1,
                inode: 1,
                size_bytes: 1,
            },
        )
        .unwrap_err();

    assert!(error.message.contains("changed identity"));
}

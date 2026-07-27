#![cfg(unix)]

use std::os::unix::fs::symlink;
use std::os::unix::net::UnixListener;

use super::*;

#[test]
fn tree_size_walks_nested_regular_files_and_tolerates_a_missing_root() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("first"), b"123").unwrap();
    std::fs::create_dir(temp.path().join("nested")).unwrap();
    std::fs::write(temp.path().join("nested/second"), b"45678").unwrap();
    assert_eq!(tree_size(temp.path()).unwrap(), 8);
    assert_eq!(tree_size(&temp.path().join("missing")).unwrap(), 0);
}

#[test]
fn tree_size_rejects_symlinks_non_regular_entries_and_non_directories() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("target");
    std::fs::write(&target, b"value").unwrap();
    symlink(&target, temp.path().join("link")).unwrap();
    assert!(tree_size(temp.path())
        .unwrap_err()
        .message
        .contains("symlink"));
    std::fs::remove_file(temp.path().join("link")).unwrap();

    let socket = temp.path().join("socket");
    let _listener = UnixListener::bind(&socket).unwrap();
    assert!(tree_size(temp.path())
        .unwrap_err()
        .message
        .contains("non-regular"));
    assert!(tree_size(&target).is_err());
}

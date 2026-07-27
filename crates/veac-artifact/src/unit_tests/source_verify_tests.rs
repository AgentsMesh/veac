#![cfg(unix)]

use std::os::unix::fs::symlink;

use crate::{source::verify_with, verify_source, ArtifactErrorKind};

#[test]
fn streaming_verification_returns_identity_and_size_without_materializing_a_large_payload() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("large");
    let size = 8 * 1024 * 1024;
    std::fs::File::create(&source)
        .unwrap()
        .set_len(size)
        .unwrap();

    let first = verify_source(&source, None).unwrap();
    assert_eq!(first.size_bytes, size);
    let second = verify_source(&source, Some(&first.identity)).unwrap();
    assert_eq!(second, first);
}

#[test]
fn streaming_verification_rejects_symlinks_and_fifos_without_blocking() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let linked = temp.path().join("linked");
    let fifo = temp.path().join("fifo");
    std::fs::write(&source, b"source").unwrap();
    symlink(&source, &linked).unwrap();

    let error = verify_source(&linked, None).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert!(std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap()
        .success());
    let error = verify_source(&fifo, None).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
}

#[test]
fn streaming_verification_fails_closed_when_the_open_path_is_swapped() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let original = temp.path().join("original");
    std::fs::write(&source, b"version-a").unwrap();
    let expected = verify_source(&source, None).unwrap().identity;

    let error = verify_with(&source, Some(&expected), || {
        std::fs::rename(&source, &original).unwrap();
        std::fs::write(&source, b"version-b").unwrap();
    })
    .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
    assert_eq!(std::fs::read(source).unwrap(), b"version-b");
}

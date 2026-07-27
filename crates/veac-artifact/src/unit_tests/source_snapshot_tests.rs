#![cfg(unix)]

use std::fs::{FileTimes, OpenOptions};
use std::os::unix::fs::{symlink, MetadataExt};
use std::time::Duration;

use veac_ir::{HashAlgorithm, MediaIdentity};

use crate::{copy_verified_source, read_verified_source, source::copy_with, ArtifactErrorKind};

#[test]
fn verified_copy_fails_closed_when_the_open_path_is_swapped() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let original = temp.path().join("original");
    let destination = temp.path().join("snapshot");
    std::fs::write(&source, b"version-a").unwrap();
    let expected = identity(b"version-a");

    let error = copy_with(&source, &destination, Some(&expected), || {
        std::fs::rename(&source, &original).unwrap();
        std::fs::write(&source, b"version-b").unwrap();
    })
    .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
    assert!(!destination.exists());
}

#[test]
fn verified_copy_rejects_in_place_changes_and_removes_partial_output() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let destination = temp.path().join("snapshot");
    std::fs::write(&source, vec![1_u8; 128 * 1024]).unwrap();
    let expected = identity(&vec![1_u8; 128 * 1024]);

    let error = copy_with(&source, &destination, Some(&expected), || {
        std::fs::write(&source, vec![2_u8; 128 * 1024]).unwrap();
    })
    .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
    assert!(!destination.exists());
}

#[test]
fn unidentified_copy_rejects_same_size_writes_with_restored_mtime() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let destination = temp.path().join("snapshot");
    std::fs::write(&source, b"version-a").unwrap();
    let before = std::fs::metadata(&source).unwrap();
    let times = FileTimes::new()
        .set_accessed(before.accessed().unwrap())
        .set_modified(before.modified().unwrap());

    let error = copy_with(&source, &destination, None, || {
        std::thread::sleep(Duration::from_millis(2));
        std::fs::write(&source, b"version-b").unwrap();
        OpenOptions::new()
            .write(true)
            .open(&source)
            .unwrap()
            .set_times(times)
            .unwrap();
        let after = std::fs::metadata(&source).unwrap();
        assert_eq!(after.len(), before.len());
        assert_eq!(after.modified().unwrap(), before.modified().unwrap());
        assert_ne!(
            (after.ctime(), after.ctime_nsec()),
            (before.ctime(), before.ctime_nsec())
        );
    })
    .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
    assert!(!destination.exists());
}

#[test]
fn unidentified_copy_is_verified_and_independent_of_the_source_path() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let destination = temp.path().join("snapshot");
    std::fs::write(&source, b"version-a").unwrap();

    let copied = copy_verified_source(&source, &destination, None).unwrap();
    std::fs::write(&source, b"version-b").unwrap();

    assert_eq!(copied.identity, identity(b"version-a"));
    assert_eq!(std::fs::read(destination).unwrap(), b"version-a");
}

#[test]
fn failed_copy_never_deletes_an_existing_destination() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let destination = temp.path().join("snapshot");
    std::fs::write(&source, b"source").unwrap();
    std::fs::write(&destination, b"sentinel").unwrap();

    assert!(copy_verified_source(&source, &destination, None).is_err());
    assert_eq!(std::fs::read(destination).unwrap(), b"sentinel");
}

#[test]
fn destination_created_during_copy_is_never_removed_or_replaced() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let destination = temp.path().join("snapshot");
    std::fs::write(&source, b"source-a").unwrap();
    let expected = identity(b"source-a");

    let error = copy_with(&source, &destination, Some(&expected), || {
        std::fs::write(&destination, b"sentinel").unwrap();
    })
    .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::Io);
    assert_eq!(std::fs::read(destination).unwrap(), b"sentinel");
}

#[test]
fn failed_copy_never_unlinks_an_external_destination_target() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let external = temp.path().join("external");
    let destination = temp.path().join("snapshot");
    std::fs::write(&source, b"source").unwrap();
    std::fs::write(&external, b"external").unwrap();
    let expected = identity(b"source");

    let error = copy_with(&source, &destination, Some(&expected), || {
        symlink(&external, &destination).unwrap();
    })
    .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::Io);
    assert!(std::fs::symlink_metadata(destination)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(std::fs::read(external).unwrap(), b"external");
}

#[test]
fn verified_sources_reject_symlinks_fifos_and_wrong_identities() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let linked = temp.path().join("linked");
    let fifo = temp.path().join("fifo");
    let directory = temp.path().join("directory");
    std::fs::write(&source, b"source").unwrap();
    symlink(&source, &linked).unwrap();
    let error = read_verified_source(&linked, None).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);

    let status = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .unwrap();
    assert!(status.success());
    let error = read_verified_source(&fifo, None).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);

    std::fs::create_dir(&directory).unwrap();
    let error = read_verified_source(&directory, None).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);

    let wrong = identity(b"wrong");
    let destination = temp.path().join("snapshot");
    let error = copy_verified_source(&source, &destination, Some(&wrong)).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
    assert!(!destination.exists());
}

fn identity(bytes: &[u8]) -> MediaIdentity {
    let digest = crate::ContentDigest::sha256(bytes);
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: digest.value,
    }
}

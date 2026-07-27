use std::io::Write;

use super::*;
use crate::{ArtifactCommitState, ContentDigest};

#[test]
fn seal_hashes_the_owned_file_not_a_replacement_visible_path() {
    let temp = tempfile::tempdir().unwrap();
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"owned").unwrap();
    let visible_directory = staged.path().parent().unwrap().to_owned();
    let moved_directory = temp.path().join("moved-stage");
    std::fs::rename(&visible_directory, &moved_directory).unwrap();
    std::fs::create_dir(&visible_directory).unwrap();
    std::fs::write(visible_directory.join("payload"), b"foreign").unwrap();

    let sealed = staged.seal().unwrap();

    assert_eq!(sealed.sha256, ContentDigest::sha256(b"owned").value);
    assert_eq!(sealed.size_bytes, 5);
    let error = staged.discard().unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(
        std::fs::read(visible_directory.join("payload")).unwrap(),
        b"foreign"
    );
}

#[test]
fn sealed_content_change_before_rename_is_not_committed() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("output");
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"owned").unwrap();
    let payload = staged.path().to_owned();
    staged.seal().unwrap();

    let error = staged
        .persist(
            &destination,
            PublishMode::Replace,
            |_| std::fs::write(&payload, b"tampered").unwrap(),
            |_| {},
        )
        .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
    assert_eq!(error.commit_state, ArtifactCommitState::NotCommitted);
    assert!(!destination.exists());
}

#[test]
fn content_change_after_rename_reports_uncertain_commit_state() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("output");
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"owned").unwrap();
    staged.seal().unwrap();

    let error = staged
        .persist(
            &destination,
            PublishMode::Replace,
            |_| {},
            |published| std::fs::write(published, b"tampered").unwrap(),
        )
        .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
    assert_eq!(error.commit_state, ArtifactCommitState::Committed);
    assert!(error.to_string().contains("destination state is uncertain"));
    assert_eq!(std::fs::read(destination).unwrap(), b"tampered");
}

#[test]
fn preexisting_payload_hard_link_prevents_publication() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("output");
    let alias = temp.path().join("alias");
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"owned").unwrap();
    std::fs::hard_link(staged.path(), &alias).unwrap();

    let error = staged.persist_replace(&destination).unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(error.commit_state, ArtifactCommitState::NotCommitted);
    assert!(!destination.exists());
    assert_eq!(std::fs::read(alias).unwrap(), b"owned");
}

#[test]
fn hard_link_created_after_rename_reports_uncertain_commit_state() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("output");
    let alias = temp.path().join("alias");
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"owned").unwrap();
    staged.seal().unwrap();

    let error = staged
        .persist(
            &destination,
            PublishMode::Replace,
            |_| {},
            |published| std::fs::hard_link(published, &alias).unwrap(),
        )
        .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(error.commit_state, ArtifactCommitState::Committed);
    assert_eq!(std::fs::read(destination).unwrap(), b"owned");
    assert_eq!(std::fs::read(alias).unwrap(), b"owned");
}

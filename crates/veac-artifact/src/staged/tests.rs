use std::io::Write;

use super::*;
use crate::ArtifactCommitState;

mod adversarial;

#[test]
fn staged_files_replace_or_publish_without_clobbering_as_requested() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("output");
    std::fs::write(&destination, b"old").unwrap();
    let mut replacement = OwnedStagedFile::new_in(temp.path()).unwrap();
    replacement.file_mut().write_all(b"new").unwrap();
    replacement.persist_replace(&destination).unwrap();
    assert_eq!(std::fs::read(&destination).unwrap(), b"new");

    let mut no_clobber = OwnedStagedFile::new_in(temp.path()).unwrap();
    no_clobber.file_mut().write_all(b"other").unwrap();
    let failed_stage = no_clobber.path().to_owned();
    assert!(no_clobber.persist_noclobber(&destination).is_err());
    assert_eq!(std::fs::read(destination).unwrap(), b"new");
    assert!(!failed_stage.exists());

    let destination = temp.path().join("new-output");
    let mut no_clobber = OwnedStagedFile::new_in(temp.path()).unwrap();
    no_clobber.file_mut().write_all(b"published").unwrap();
    no_clobber.persist_noclobber(&destination).unwrap();
    assert_eq!(std::fs::read(destination).unwrap(), b"published");
}

#[test]
fn replaced_stage_paths_are_never_published_or_unlinked() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("output");
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"owned").unwrap();
    let path = staged.path().to_owned();
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&path, b"foreign").unwrap();

    let error = staged.persist_noclobber(&destination).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(error.commit_state, ArtifactCommitState::NotCommitted);
    assert_eq!(std::fs::read(path).unwrap(), b"foreign");
    assert!(!destination.exists());
}

#[test]
fn owned_stage_discard_removes_only_the_verified_entry() {
    let temp = tempfile::tempdir().unwrap();
    let staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    let path = staged.path().to_owned();
    staged.discard().unwrap();
    assert!(!path.exists());
}

#[test]
fn staged_payload_path_can_be_reverified_before_publication() {
    let temp = tempfile::tempdir().unwrap();
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"verified").unwrap();
    staged.verify_path().unwrap();
    staged.discard().unwrap();
}

#[test]
fn drop_removes_an_owned_stage() {
    let temp = tempfile::tempdir().unwrap();
    let path = {
        let staged = OwnedStagedFile::new_in(temp.path()).unwrap();
        staged.path().to_owned()
    };
    assert!(!path.exists());
}

#[test]
fn drop_preserves_a_replaced_stage_entry() {
    let temp = tempfile::tempdir().unwrap();
    let staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    let path = staged.path().to_owned();
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&path, b"foreign").unwrap();

    drop(staged);

    assert_eq!(std::fs::read(path).unwrap(), b"foreign");
}

#[test]
fn publication_is_bound_to_the_open_private_directory() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("output");
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"owned").unwrap();
    let visible_directory = staged.path().parent().unwrap().to_owned();
    let moved_directory = temp.path().join("moved-stage");
    let replacement_payload = visible_directory.join("payload");

    let error = staged
        .persist(
            &destination,
            PublishMode::Replace,
            |_| {
                std::fs::rename(&visible_directory, &moved_directory).unwrap();
                std::fs::create_dir(&visible_directory).unwrap();
                std::fs::write(&replacement_payload, b"foreign").unwrap();
            },
            |_| {},
        )
        .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(error.commit_state, ArtifactCommitState::Committed);
    assert_eq!(std::fs::read(destination).unwrap(), b"owned");
    assert_eq!(std::fs::read(replacement_payload).unwrap(), b"foreign");
    assert!(std::fs::read_dir(moved_directory).unwrap().next().is_none());
}

#[test]
fn post_publish_replacement_fails_without_deleting_the_foreign_output() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("output");
    let mut staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    staged.file_mut().write_all(b"owned").unwrap();

    let error = staged
        .persist(
            &destination,
            PublishMode::Replace,
            |_| {},
            |published| {
                std::fs::remove_file(published).unwrap();
                std::fs::write(published, b"foreign").unwrap();
            },
        )
        .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(error.commit_state, ArtifactCommitState::Committed);
    assert_eq!(std::fs::read(destination).unwrap(), b"foreign");
    assert!(stage_directories(temp.path()).is_empty());
}

#[test]
fn discard_reports_a_replaced_private_directory() {
    let temp = tempfile::tempdir().unwrap();
    let staged = OwnedStagedFile::new_in(temp.path()).unwrap();
    let visible_directory = staged.path().parent().unwrap().to_owned();
    let moved_directory = temp.path().join("moved-stage");
    std::fs::rename(&visible_directory, &moved_directory).unwrap();
    std::fs::create_dir(&visible_directory).unwrap();
    std::fs::write(visible_directory.join("foreign"), b"foreign").unwrap();

    let error = staged.discard().unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(
        std::fs::read(visible_directory.join("foreign")).unwrap(),
        b"foreign"
    );
    assert!(std::fs::read_dir(moved_directory).unwrap().next().is_none());
}

fn stage_directories(parent: &Path) -> Vec<std::path::PathBuf> {
    std::fs::read_dir(parent)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| {
            path.file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with(".veac-stage-")
        })
        .collect()
}

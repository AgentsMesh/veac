use std::path::Path;

use super::super::commit::{self, CommitContext};
use super::super::directory::Directory;
use super::super::StagedFile;
use super::{apply, deadline, file_outputs};

#[test]
fn commit_replaces_targets_and_removes_stale_files() {
    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let first = staged(staging.path(), "first", temp.path(), "a.out", b"new-a");
    let second = staged(staging.path(), "second", temp.path(), "b.out", b"new-b");
    let stale = temp.path().join("stale.out");
    std::fs::write(&first.target, b"old-a").unwrap();
    std::fs::write(&stale, b"old-stale").unwrap();

    apply(
        staging.path(),
        &[first.clone(), second.clone()],
        std::slice::from_ref(&stale),
        false,
    )
    .unwrap();
    assert_eq!(std::fs::read(first.target).unwrap(), b"new-a");
    assert_eq!(std::fs::read(second.target).unwrap(), b"new-b");
    assert!(!stale.exists());
}

#[test]
fn commit_validation_rejects_duplicate_empty_missing_and_overlapping_files() {
    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let file = staged(staging.path(), "source", temp.path(), "output", b"value");
    let duplicate = StagedFile {
        source: file.source.clone(),
        target: file.target.clone(),
        allow_empty: false,
    };
    assert!(
        apply(staging.path(), &[file.clone(), duplicate], &[], false)
            .unwrap_err()
            .message
            .contains("duplicate output targets")
    );
    assert!(apply(
        staging.path(),
        std::slice::from_ref(&file),
        std::slice::from_ref(&file.target),
        false
    )
    .unwrap_err()
    .message
    .contains("stale and current"));

    let missing = StagedFile {
        source: staging.path().join("missing"),
        target: temp.path().join("missing.out"),
        allow_empty: false,
    };
    assert!(apply(staging.path(), &[missing], &[], false)
        .unwrap_err()
        .message
        .contains("atomic render output commit failed"));

    let empty = staged(staging.path(), "empty", temp.path(), "empty.out", b"");
    assert!(apply(staging.path(), std::slice::from_ref(&empty), &[], false).is_err());
    apply(staging.path(), &[empty], &[], true).unwrap();
}

#[test]
fn empty_permission_is_scoped_to_each_staged_file() {
    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let caption = staged(staging.path(), "caption", temp.path(), "caption.srt", b"");
    let mut video = staged(staging.path(), "video", temp.path(), "video.mp4", b"");
    let mut caption = caption;
    caption.allow_empty = true;
    video.allow_empty = false;
    let stage = Directory::open(staging.path()).unwrap();
    let output = Directory::open(temp.path()).unwrap();

    let outputs = file_outputs(&[caption, video]);
    let error = commit::apply_locked(
        CommitContext::new(staging.path(), &stage, &output, &outputs, &[]),
        deadline(),
    )
    .unwrap_err()
    .into_error();

    assert!(error.message.contains("regular non-empty file"));
    assert!(!temp.path().join("caption.srt").exists());
    assert!(!temp.path().join("video.mp4").exists());
}

#[test]
fn unsafe_later_target_rolls_back_an_earlier_backup() {
    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let first = staged(staging.path(), "first", temp.path(), "a.out", b"new-a");
    let second = staged(staging.path(), "second", temp.path(), "b.out", b"new-b");
    std::fs::write(&first.target, b"old-a").unwrap();
    std::fs::create_dir(&second.target).unwrap();
    let error = apply(staging.path(), &[first.clone(), second], &[], false).unwrap_err();
    assert!(
        error
            .message
            .contains("must be an exclusively linked regular file"),
        "{error}"
    );
    assert_eq!(std::fs::read(first.target).unwrap(), b"old-a");
}

#[test]
fn failed_second_install_removes_new_files_and_restores_backups() {
    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let source = staging.path().join("shared");
    std::fs::write(&source, b"new").unwrap();
    let first_target = temp.path().join("a.out");
    let second_target = temp.path().join("b.out");
    std::fs::write(&first_target, b"old-a").unwrap();
    std::fs::write(&second_target, b"old-b").unwrap();
    let files = [
        StagedFile {
            source: source.clone(),
            target: first_target.clone(),
            allow_empty: false,
        },
        StagedFile {
            source,
            target: second_target.clone(),
            allow_empty: false,
        },
    ];
    let error = apply(staging.path(), &files, &[], false).unwrap_err();
    assert!(error.message.contains("changed identity"), "{error}");
    assert_eq!(std::fs::read(first_target).unwrap(), b"old-a");
    assert_eq!(std::fs::read(second_target).unwrap(), b"old-b");
}

#[cfg(unix)]
#[test]
fn staged_symlink_is_rejected_before_commit() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let source = staging.path().join("source");
    let link = staging.path().join("link");
    std::fs::write(&source, b"value").unwrap();
    symlink(source, &link).unwrap();
    let error = apply(
        staging.path(),
        &[StagedFile {
            source: link,
            target: temp.path().join("output"),
            allow_empty: false,
        }],
        &[],
        false,
    )
    .unwrap_err();
    assert!(error.message.contains("regular non-empty file"));
}

fn staged(
    staging: &Path,
    source: &str,
    target_parent: &Path,
    target: &str,
    content: &[u8],
) -> StagedFile {
    let source = staging.join(source);
    std::fs::write(&source, content).unwrap();
    StagedFile {
        source,
        target: target_parent.join(target),
        allow_empty: false,
    }
}

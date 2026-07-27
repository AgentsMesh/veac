use std::path::{Path, PathBuf};

use super::super::directory::Directory;
use super::super::StagedFile;
use super::{commit_journal, prepare_journal, recover};

#[test]
fn prepared_journal_before_any_rename_preserves_the_original() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "before-rename");
    let file = staged(&staging, "source", temp.path(), "output", b"new");
    std::fs::write(&file.target, b"old").unwrap();
    prepare_journal(&staging, std::slice::from_ref(&file), &[]).unwrap();

    recover(temp.path()).unwrap();

    assert_eq!(std::fs::read(file.target).unwrap(), b"old");
    assert!(!staging.exists());
}

#[test]
fn prepared_journal_rejects_a_backup_for_a_new_output() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "unexpected-backup");
    let file = staged(&staging, "source", temp.path(), "output", b"new");
    prepare_journal(&staging, std::slice::from_ref(&file), &[]).unwrap();
    let backup = staging.join("backups/0");
    std::fs::write(&backup, b"unexpected").unwrap();

    let error = recover(temp.path()).unwrap_err();

    assert!(error
        .message
        .contains("new output unexpectedly has a recovery backup"));
    assert!(!file.target.exists());
    assert_eq!(std::fs::read(backup).unwrap(), b"unexpected");
}

#[test]
fn committed_journal_rejects_a_missing_installed_output() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "missing-committed-output");
    let file = staged(&staging, "source", temp.path(), "output", b"new");
    let mut transaction = prepare_journal(&staging, std::slice::from_ref(&file), &[]).unwrap();
    commit_journal(&staging, &mut transaction).unwrap();

    let error = recover(temp.path()).unwrap_err();

    assert!(error.message.contains("changed identity"), "{error}");
    assert!(staging.exists());
}

#[test]
fn committed_journal_rejects_a_stale_output_that_reappeared() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "reappeared-stale-output");
    let file = staged(&staging, "source", temp.path(), "master", b"new");
    let stale = temp.path().join("stale");
    std::fs::write(&stale, b"stale").unwrap();
    let mut transaction = prepare_journal(
        &staging,
        std::slice::from_ref(&file),
        std::slice::from_ref(&stale),
    )
    .unwrap();
    std::fs::rename(&file.source, &file.target).unwrap();
    commit_journal(&staging, &mut transaction).unwrap();

    let error = recover(temp.path()).unwrap_err();

    assert!(error
        .message
        .contains("committed stale output still exists"));
    assert_eq!(std::fs::read(file.target).unwrap(), b"new");
    assert_eq!(std::fs::read(stale).unwrap(), b"stale");
}

#[test]
fn recovery_refuses_a_non_directory_reserved_stage_path() {
    let temp = tempfile::tempdir().unwrap();
    let reserved = temp.path().join(".veac-stage-reserved");
    std::fs::write(&reserved, b"untrusted").unwrap();

    let error = recover(temp.path()).unwrap_err();

    assert!(error.message.contains("non-symlink directory"));
    assert_eq!(std::fs::read(reserved).unwrap(), b"untrusted");
}

#[test]
fn recovery_leaves_an_unjournaled_stage_directory_untouched() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "unjournaled");
    let payload = staging.join("payload");
    std::fs::write(&payload, b"in-progress").unwrap();

    recover(temp.path()).unwrap();

    assert_eq!(std::fs::read(payload).unwrap(), b"in-progress");
    assert!(staging.join("backups").is_dir());
}

#[test]
fn descriptor_cleanup_never_recursively_removes_a_replacement_stage() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "replaced-cleanup");
    let moved = temp.path().join("moved-stage");
    std::fs::write(staging.join("payload"), b"owned").unwrap();
    let output = Directory::open(temp.path()).unwrap();
    let descriptor = Directory::open(&staging).unwrap();
    std::fs::rename(&staging, &moved).unwrap();
    std::fs::create_dir(&staging).unwrap();
    std::fs::write(staging.join("sentinel"), b"foreign").unwrap();

    let error = super::super::recovery::discard_bound_stage(
        &output,
        ".veac-stage-replaced-cleanup",
        &descriptor,
        super::deadline(),
    )
    .unwrap_err();

    assert!(error.message.contains("changed identity"), "{error}");
    assert_eq!(std::fs::read(staging.join("sentinel")).unwrap(), b"foreign");
    assert!(std::fs::read_dir(moved).unwrap().next().is_none());
}

fn stage(parent: &Path, name: &str) -> PathBuf {
    let path = parent.join(format!(".veac-stage-{name}"));
    std::fs::create_dir(&path).unwrap();
    std::fs::create_dir(path.join("backups")).unwrap();
    path
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
    }
}

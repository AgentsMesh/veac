use std::path::{Path, PathBuf};

use super::super::{journal, StagedFile};
use super::{apply, load_journal, prepare_journal, recover};

#[test]
fn prepared_journal_rolls_back_partial_multi_file_commit() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path());
    let old = staged(&staging, "old-source", temp.path(), "old.out", b"new-old");
    let new = staged(&staging, "new-source", temp.path(), "new.out", b"new");
    let stale = temp.path().join("stale.out");
    std::fs::write(&old.target, b"old").unwrap();
    std::fs::write(&stale, b"stale").unwrap();
    let transaction = prepare_journal(
        &staging,
        &[old.clone(), new.clone()],
        std::slice::from_ref(&stale),
    )
    .unwrap();
    simulate_partial_commit(temp.path(), &staging, &transaction);

    recover(temp.path()).unwrap();
    assert_eq!(std::fs::read(old.target).unwrap(), b"old");
    assert_eq!(std::fs::read(stale).unwrap(), b"stale");
    assert!(!new.target.exists());
    assert!(!staging.exists());
}

#[test]
fn committed_journal_finishes_cleanup_without_rolling_back_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path());
    let file = staged(&staging, "source", temp.path(), "master.out", b"new");
    std::fs::write(&file.target, b"old").unwrap();
    apply(&staging, std::slice::from_ref(&file), &[], false).unwrap();
    assert_eq!(
        load_journal(&staging).unwrap().state,
        journal::JournalState::Committed
    );

    recover(temp.path()).unwrap();
    assert_eq!(std::fs::read(file.target).unwrap(), b"new");
    assert!(!staging.exists());
}

#[test]
fn malformed_journal_fails_closed_without_touching_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path());
    let victim = temp.path().join("victim");
    std::fs::write(&victim, b"unchanged").unwrap();
    std::fs::write(
        staging.join(journal::JOURNAL_NAME),
        br#"{"entries":[{"original":null,"source":"source","source_identity":{"device":1,"inode":1,"node_type":"regular","size_bytes":3},"target":"../victim"}],"schema_version":3,"state":"prepared"}"#,
    )
    .unwrap();
    let error = recover(temp.path()).unwrap_err();
    assert!(error.message.contains("single-component"));
    assert_eq!(std::fs::read(victim).unwrap(), b"unchanged");
}

#[cfg(unix)]
#[test]
fn recovery_refuses_a_target_symlink_created_after_prepare() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path());
    let file = staged(&staging, "source", temp.path(), "output", b"new");
    std::fs::create_dir(staging.join("backups")).unwrap();
    prepare_journal(&staging, std::slice::from_ref(&file), &[]).unwrap();
    let victim = temp.path().join("victim");
    std::fs::write(&victim, b"unchanged").unwrap();
    symlink(&victim, &file.target).unwrap();

    let error = recover(temp.path()).unwrap_err();
    assert!(
        error.message.contains("exclusively linked regular file"),
        "{error}"
    );
    assert_eq!(std::fs::read(victim).unwrap(), b"unchanged");
}

fn simulate_partial_commit(parent: &Path, staging: &Path, transaction: &journal::Journal) {
    let backups = staging.join("backups");
    std::fs::create_dir(&backups).unwrap();
    for (index, entry) in transaction.entries.iter().enumerate() {
        let target = parent.join(&entry.target);
        if entry.original.is_some() {
            std::fs::rename(&target, backups.join(index.to_string())).unwrap();
        }
        if let Some(source) = &entry.source {
            std::fs::rename(staging.join(source), target).unwrap();
        }
    }
}

fn stage(parent: &Path) -> PathBuf {
    let path = parent.join(".veac-stage-recovery-test");
    std::fs::create_dir(&path).unwrap();
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
        allow_empty: false,
    }
}

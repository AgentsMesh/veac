use std::path::{Path, PathBuf};

use super::super::{journal, StagedFile};
use super::{load_journal, recover, task, RollbackFault};

#[test]
fn failed_rollback_remove_preserves_prepared_recovery_state() {
    let value = Case::new(false);
    let error = value
        .task
        .commit_with_fault(&value.locks, RollbackFault::Remove)
        .unwrap_err();

    assert_failure(&error, "remove");
    assert_prepared(&value.staging);
    assert_eq!(std::fs::read(&value.first).unwrap(), b"new");
    assert!(!value.second.exists());

    drop(value.locks);
    recover(value.root.path()).unwrap();
    assert!(!value.first.exists());
    assert!(!value.second.exists());
    assert!(!value.staging.exists());
}

#[test]
fn failed_rollback_restore_preserves_backups_for_startup_recovery() {
    let value = Case::new(true);
    let error = value
        .task
        .commit_with_fault(&value.locks, RollbackFault::Restore)
        .unwrap_err();

    assert_failure(&error, "restore");
    assert_prepared(&value.staging);
    assert_eq!(
        std::fs::read_dir(value.staging.join("backups"))
            .unwrap()
            .count(),
        2
    );
    assert!(!value.first.exists());
    assert!(!value.second.exists());

    drop(value.locks);
    recover(value.root.path()).unwrap();
    assert_eq!(std::fs::read(&value.first).unwrap(), b"old-a");
    assert_eq!(std::fs::read(&value.second).unwrap(), b"old-b");
    assert!(!value.staging.exists());
}

struct Case {
    root: tempfile::TempDir,
    staging: PathBuf,
    first: PathBuf,
    second: PathBuf,
    locks: crate::executor::locking::OutputLocks,
    task: super::super::StagedTask,
}

impl Case {
    fn new(originals: bool) -> Self {
        let root = tempfile::tempdir().unwrap();
        let directory = tempfile::Builder::new()
            .prefix(".veac-stage-")
            .tempdir_in(root.path())
            .unwrap();
        let staging = directory.path().to_owned();
        let source = staging.join("shared");
        std::fs::write(&source, b"new").unwrap();
        let first = root.path().join("a.out");
        let second = root.path().join("b.out");
        if originals {
            std::fs::write(&first, b"old-a").unwrap();
            std::fs::write(&second, b"old-b").unwrap();
        }
        let files = [first.clone(), second.clone()]
            .into_iter()
            .map(|target| StagedFile {
                source: source.clone(),
                target,
            })
            .collect();
        let task = task(directory, files);
        let parent = std::fs::canonicalize(root.path()).unwrap();
        let locks = crate::executor::locking::acquire_until(&[parent], super::deadline()).unwrap();
        Self {
            root,
            staging,
            first,
            second,
            locks,
            task,
        }
    }
}

fn assert_failure(error: &crate::RuntimeError, operation: &str) {
    assert!(error.message.contains("changed identity"), "{error}");
    assert!(
        error.message.contains(&format!(
            "rollback failed: injected rollback {operation} failure"
        )),
        "{error}"
    );
    assert!(error.message.contains("recovery staging preserved at"));
}

fn assert_prepared(staging: &Path) {
    assert!(staging.exists());
    assert!(staging.join("backups").is_dir());
    assert_eq!(
        load_journal(staging).unwrap().state,
        journal::JournalState::Prepared
    );
}

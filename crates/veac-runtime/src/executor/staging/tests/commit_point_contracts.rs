use std::path::Path;

use super::super::commit::{CommitContext, CommitObserver};
use super::super::directory::Directory;
use super::super::{journal, StagedFile};
use super::{apply_observed, file_outputs, load_journal, recover, RollbackFault};
use crate::RuntimeError;

#[test]
fn committed_rename_failure_preserves_the_complete_new_bundle() {
    let root = tempfile::tempdir().unwrap();
    let staging = root.path().join(".veac-stage-commit-point");
    std::fs::create_dir(&staging).unwrap();
    let files = [
        staged(&staging, root.path(), "a", b"new-a", b"old-a"),
        staged(&staging, root.path(), "b", b"new-b", b"old-b"),
    ];
    let stage = Directory::open(&staging).unwrap();
    let output = Directory::open(root.path()).unwrap();

    let outputs = file_outputs(&files);
    let failure = apply_observed(
        CommitContext::new(&staging, &stage, &output, &outputs, &[]),
        || true,
        &RollbackFault::None,
        &FailCommittedSync,
    )
    .unwrap_err();

    assert!(failure.preserve_staging());
    assert!(failure.crossed_commit());
    assert_eq!(
        load_journal(&staging).unwrap().state,
        journal::JournalState::Committed
    );
    assert_new(&files);
    drop(stage);
    drop(output);
    recover(root.path()).unwrap();
    assert_new(&files);
    assert!(!staging.exists());
}

struct FailCommittedSync;

impl CommitObserver for FailCommittedSync {
    fn sync_committed(&self, _stage: &Directory) -> Result<(), RuntimeError> {
        Err(RuntimeError::new("injected committed sync failure"))
    }
}

fn staged(staging: &Path, output: &Path, name: &str, new: &[u8], old: &[u8]) -> StagedFile {
    let source = staging.join(name);
    let target = output.join(format!("{name}.out"));
    std::fs::write(&source, new).unwrap();
    std::fs::write(&target, old).unwrap();
    StagedFile {
        source,
        target,
        allow_empty: false,
    }
}

fn assert_new(files: &[StagedFile]) {
    assert_eq!(std::fs::read(&files[0].target).unwrap(), b"new-a");
    assert_eq!(std::fs::read(&files[1].target).unwrap(), b"new-b");
}

use std::path::{Path, PathBuf};

use super::super::commit::{self, CommitContext, CommitObserver};
use super::super::directory::Directory;
use super::super::StagedFile;
use super::{load_journal, RollbackFault};

#[test]
fn source_swap_at_the_rename_boundary_never_commits_the_journal() {
    let root = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(root.path()).unwrap();
    let source = staging.path().join("payload");
    let moved = staging.path().join("owned-payload");
    let target = root.path().join("output");
    std::fs::write(&source, b"owned").unwrap();
    let files = [StagedFile {
        source: source.clone(),
        target: target.clone(),
    }];
    let stage = Directory::open(staging.path()).unwrap();
    let output = Directory::open(root.path()).unwrap();
    let observer = SwapSource {
        source,
        moved: moved.clone(),
    };

    let failure = commit::apply_observed(
        CommitContext::new(staging.path(), &stage, &output, &files, &[], false),
        || true,
        &RollbackFault::None,
        &observer,
    )
    .unwrap_err();

    assert!(failure.preserve_staging());
    assert!(failure.into_error().message.contains("rollback failed"));
    assert_eq!(std::fs::read(&target).unwrap(), b"foreign");
    assert_eq!(std::fs::read(moved).unwrap(), b"owned");
    assert_eq!(
        load_journal(staging.path()).unwrap().state,
        super::super::journal::JournalState::Prepared
    );
}

struct SwapSource {
    source: PathBuf,
    moved: PathBuf,
}

impl CommitObserver for SwapSource {
    fn before_install(&self, source: &str, _target: &str) {
        assert_eq!(Path::new(source), self.source.file_name().unwrap());
        std::fs::rename(&self.source, &self.moved).unwrap();
        std::fs::write(&self.source, b"foreign").unwrap();
    }
}

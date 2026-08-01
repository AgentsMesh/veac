use std::path::{Path, PathBuf};

use super::super::directory::Directory;
use super::super::ownership::{self, MARKER_NAME};
use super::{prepare_journal, recover, StagedFile};

#[test]
fn recovery_discards_an_owned_unjournaled_stage() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "owned");
    mark(&staging, ".veac-stage-owned");
    std::fs::create_dir(staging.join("preparations")).unwrap();
    std::fs::write(staging.join("preparations/analysis.trf"), b"analysis").unwrap();

    recover(temp.path()).unwrap();

    assert!(!staging.exists());
}

#[test]
fn recovery_preserves_an_unmarked_or_malformed_stage() {
    let temp = tempfile::tempdir().unwrap();
    let unmarked = stage(temp.path(), "unmarked");
    let malformed = stage(temp.path(), "malformed");
    std::fs::write(malformed.join(MARKER_NAME), b"not a VEAC marker").unwrap();

    recover(temp.path()).unwrap();

    assert!(unmarked.exists());
    assert!(malformed.exists());
}

#[test]
fn recovery_preserves_a_marker_bound_to_another_stage() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "foreign");
    mark(&staging, ".veac-stage-somewhere-else");

    recover(temp.path()).unwrap();

    assert!(staging.exists());
}

#[test]
fn recovery_preserves_a_non_regular_marker() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "marker-directory");
    std::fs::create_dir(staging.join(MARKER_NAME)).unwrap();

    recover(temp.path()).unwrap();

    assert!(staging.join(MARKER_NAME).is_dir());
}

#[test]
fn journal_recovery_accepts_and_removes_the_owned_marker() {
    let temp = tempfile::tempdir().unwrap();
    let staging = stage(temp.path(), "journaled");
    mark(&staging, ".veac-stage-journaled");
    let file = staged(&staging, temp.path());
    std::fs::write(&file.target, b"old").unwrap();
    prepare_journal(&staging, std::slice::from_ref(&file), &[]).unwrap();

    recover(temp.path()).unwrap();

    assert_eq!(std::fs::read(file.target).unwrap(), b"old");
    assert!(!staging.exists());
}

fn stage(parent: &Path, suffix: &str) -> PathBuf {
    let staging = parent.join(format!(".veac-stage-{suffix}"));
    std::fs::create_dir(&staging).unwrap();
    std::fs::create_dir(staging.join("backups")).unwrap();
    staging
}

fn mark(staging: &Path, name: &str) {
    let descriptor = Directory::open(staging).unwrap();
    ownership::mark(&descriptor, name).unwrap();
}

fn staged(staging: &Path, parent: &Path) -> StagedFile {
    let source = staging.join("source");
    std::fs::write(&source, b"new").unwrap();
    StagedFile {
        source,
        target: parent.join("output"),
        allow_empty: false,
    }
}

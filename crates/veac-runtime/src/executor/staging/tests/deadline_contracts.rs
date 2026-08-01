use super::super::directory::Directory;
use super::super::{commit, StagedFile};
use super::{apply_with, file_outputs, RollbackFault};
use crate::RuntimeErrorKind;

#[test]
fn deadline_during_install_rolls_back_every_output() {
    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let files = [
        staged(staging.path(), temp.path(), "first", b"new-a", b"old-a"),
        staged(staging.path(), temp.path(), "second", b"new-b", b"old-b"),
    ];
    let stage = Directory::open(staging.path()).unwrap();
    let output = Directory::open(temp.path()).unwrap();
    let mut calls = 0;
    let outputs = file_outputs(&files);
    let error = apply_with(
        commit::CommitContext::new(staging.path(), &stage, &output, &outputs, &[]),
        || {
            calls += 1;
            calls < 11
        },
        &RollbackFault::None,
    )
    .unwrap_err()
    .into_error();
    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
    assert_eq!(calls, 11);
    assert_eq!(std::fs::read(&files[0].target).unwrap(), b"old-a");
    assert_eq!(std::fs::read(&files[1].target).unwrap(), b"old-b");
}

fn staged(
    staging: &std::path::Path,
    output: &std::path::Path,
    name: &str,
    new: &[u8],
    old: &[u8],
) -> StagedFile {
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

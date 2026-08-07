use rustix::fd::OwnedFd;
use rustix::fs::{flock, open, openat, FlockOperation, Mode, OFlags};

use super::*;

fn source_lock(root: &std::path::Path, operation: FlockOperation) -> OwnedFd {
    let directory = open(root, OFlags::RDONLY | OFlags::DIRECTORY, Mode::empty()).unwrap();
    let lock = openat(
        directory,
        ".veac-source.lock",
        OFlags::CREATE | OFlags::RDWR,
        Mode::RUSR | Mode::WUSR,
    )
    .unwrap();
    flock(&lock, operation).unwrap();
    lock
}

#[test]
fn source_graph_commands_reject_an_active_writer() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let _writer = source_lock(temp.path(), FlockOperation::NonBlockingLockExclusive);

    for command in ["check", "build", "source-index", "source-revision", "fmt"] {
        veac()
            .args([command, source.to_str().unwrap()])
            .assert()
            .failure()
            .stderr(predicate::str::contains("SOURCE_LOCKED"));
    }
}

#[test]
fn source_edit_rejects_an_active_reader() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, &program_source());
    let batch_path = temp.path().join("shared-lock.json");
    let edit = batch(revision(&source), "400ms");
    std::fs::write(&batch_path, serde_json::to_string_pretty(&edit).unwrap()).unwrap();
    let _reader = source_lock(temp.path(), FlockOperation::NonBlockingLockShared);

    veac()
        .args([
            "source-edit",
            source.to_str().unwrap(),
            batch_path.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("SOURCE_LOCKED"));
}

#[test]
fn build_cannot_replace_the_source_graph_lock() {
    let temp = tempdir().unwrap();
    let source = source_file(&temp, EXECUTABLE_SOURCE);
    let lock = temp.path().join(".veac-source.lock");

    veac()
        .args([
            "build",
            source.to_str().unwrap(),
            "--emit-ir",
            lock.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("OUTPUT_OVERWRITES_INPUT"));
}

use tempfile::tempdir;

use super::{SourceGraphLock, SourceGraphReadLock};

#[test]
fn readers_share_one_snapshot_lock_and_exclude_a_writer() {
    let temp = tempdir().unwrap();
    let first = SourceGraphReadLock::acquire(temp.path()).unwrap();
    first.revalidate(temp.path()).unwrap();
    let second = SourceGraphReadLock::acquire(temp.path()).unwrap();
    second.revalidate(temp.path()).unwrap();

    let error = SourceGraphLock::acquire(temp.path()).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_LOCKED");

    drop(first);
    drop(second);
    SourceGraphLock::acquire(temp.path()).unwrap();
}

#[test]
fn writer_excludes_a_source_graph_reader() {
    let temp = tempdir().unwrap();
    let writer = SourceGraphLock::acquire(temp.path()).unwrap();
    let error = SourceGraphReadLock::acquire(temp.path()).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_LOCKED");
    drop(writer);
    SourceGraphReadLock::acquire(temp.path()).unwrap();
}

#[test]
fn shared_lock_keeps_a_multimodule_read_in_one_generation() {
    let temp = tempdir().unwrap();
    let first = temp.path().join("main.veac");
    let second = temp.path().join("module.veac");
    std::fs::write(&first, "generation one").unwrap();
    std::fs::write(&second, "generation one").unwrap();

    let reader = SourceGraphReadLock::acquire(temp.path()).unwrap();
    let observed_first = std::fs::read_to_string(&first).unwrap();
    assert_eq!(
        SourceGraphLock::acquire(temp.path())
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_LOCKED"
    );
    let observed_second = std::fs::read_to_string(&second).unwrap();

    assert_eq!(observed_first, observed_second);
    reader.revalidate(temp.path()).unwrap();
}

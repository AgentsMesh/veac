use tempfile::tempdir;

use super::{SourceGraphLock, SourceGraphReadLock};

#[path = "tests/batch.rs"]
mod batch;
#[path = "tests/commit_races.rs"]
mod commit_races;
#[path = "tests/error_paths.rs"]
mod error_paths;
#[path = "tests/path_races.rs"]
mod path_races;
#[path = "tests/read.rs"]
mod read;
#[path = "tests/stage_cas.rs"]
mod stage_cas;
#[path = "tests/stage_races.rs"]
mod stage_races;

#[test]
fn source_graph_lock_is_exclusive_and_reusable() {
    let temp = tempdir().unwrap();
    let first = SourceGraphLock::acquire(temp.path()).unwrap();
    first.revalidate(temp.path()).unwrap();
    let error = SourceGraphLock::acquire(temp.path()).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_LOCKED");
    drop(first);
    SourceGraphLock::acquire(temp.path()).unwrap();
}

#[cfg(unix)]
#[test]
fn source_graph_lock_rejects_a_symlink_lock_file() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let target = temp.path().join("target");
    std::fs::write(&target, "not a lock").unwrap();
    symlink(&target, temp.path().join(".veac-source.lock")).unwrap();
    assert_eq!(
        SourceGraphLock::acquire(temp.path())
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_LOCK_FAILED"
    );
}

#[test]
fn source_graph_lock_detects_root_directory_replacement() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("source");
    std::fs::create_dir(&root).unwrap();
    let lock = SourceGraphLock::acquire(&root).unwrap();
    std::fs::rename(&root, temp.path().join("moved")).unwrap();
    std::fs::create_dir(&root).unwrap();
    assert_eq!(
        lock.revalidate(&root).unwrap_err().diagnostics()[0].code,
        "SOURCE_CHANGED"
    );
}

#[test]
fn source_graph_lock_maps_missing_root_failures() {
    let temp = tempdir().unwrap();
    let root = temp.path().join("source");
    let error = SourceGraphLock::acquire(&root).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_LOCK_FAILED");

    std::fs::create_dir(&root).unwrap();
    let lock = SourceGraphLock::acquire(&root).unwrap();
    std::fs::rename(&root, temp.path().join("moved")).unwrap();
    let error = lock.revalidate(&root).unwrap_err();
    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
}

#[test]
fn source_graph_lock_detects_its_path_being_replaced() {
    let temp = tempdir().unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    let path = temp.path().join(".veac-source.lock");
    std::fs::remove_file(&path).unwrap();
    std::fs::write(&path, b"replacement").unwrap();

    assert_eq!(
        lock.revalidate(temp.path()).unwrap_err().diagnostics()[0].code,
        "SOURCE_CHANGED"
    );
    SourceGraphLock::acquire(temp.path()).unwrap();
}

#[cfg(unix)]
#[test]
fn source_graph_lock_rejects_a_fifo_without_blocking() {
    let temp = tempdir().unwrap();
    let status = std::process::Command::new("mkfifo")
        .arg(temp.path().join(".veac-source.lock"))
        .status()
        .unwrap();
    assert!(status.success());
    assert_eq!(
        SourceGraphLock::acquire(temp.path())
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_LOCK_FAILED"
    );
}

#[test]
fn locked_commit_replaces_only_the_expected_regular_module() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();

    lock.commit_module(temp.path(), "main.veac", "before", "after")
        .unwrap();

    assert_eq!(std::fs::read_to_string(source).unwrap(), "after");
}

#[test]
fn locked_commit_rejects_stale_bytes() {
    let temp = tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "current").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    let error = lock
        .commit_module(temp.path(), "main.veac", "stale", "after")
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(std::fs::read_to_string(source).unwrap(), "current");
}

#[cfg(unix)]
#[test]
fn locked_commit_rejects_symlinks_and_fifos_without_writing_outside() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let outside = tempdir().unwrap();
    let target = outside.path().join("outside.veac");
    std::fs::write(&target, "outside").unwrap();
    symlink(&target, temp.path().join("main.veac")).unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    assert_eq!(
        lock.commit_module(temp.path(), "main.veac", "outside", "changed")
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_CHANGED"
    );
    assert_eq!(std::fs::read_to_string(&target).unwrap(), "outside");

    std::fs::remove_file(temp.path().join("main.veac")).unwrap();
    assert!(std::process::Command::new("mkfifo")
        .arg(temp.path().join("main.veac"))
        .status()
        .unwrap()
        .success());
    assert_eq!(
        lock.commit_module(temp.path(), "main.veac", "", "changed")
            .unwrap_err()
            .diagnostics()[0]
            .code,
        "SOURCE_CHANGED"
    );
}

#[cfg(unix)]
#[test]
fn locked_commit_rejects_a_symlinked_module_directory() {
    use std::os::unix::fs::symlink;

    let temp = tempdir().unwrap();
    let outside = tempdir().unwrap();
    std::fs::write(outside.path().join("brand.veac"), "outside").unwrap();
    symlink(outside.path(), temp.path().join("parts")).unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    let error = lock
        .commit_module(temp.path(), "parts/brand.veac", "outside", "changed")
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(
        std::fs::read_to_string(outside.path().join("brand.veac")).unwrap(),
        "outside"
    );
}

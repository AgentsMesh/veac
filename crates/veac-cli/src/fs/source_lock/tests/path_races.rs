use std::io::Write;

use super::super::path;
use super::SourceGraphLock;

#[test]
fn target_snapshot_rejects_an_append_after_eof() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    let parent = path::resolve(&lock.directory, "main.veac", &source).unwrap();

    let result = path::read_target_with(&parent, &source, 64, || {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .open(&source)
            .unwrap();
        file.write_all(b"-external").unwrap();
        file.sync_all().unwrap();
    });
    let Err(error) = result else {
        panic!("append after EOF must invalidate the target snapshot");
    };

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(std::fs::read_to_string(source).unwrap(), "before-external");
}

#[test]
fn target_snapshot_rejects_a_same_inode_rewrite_after_read() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    let parent = path::resolve(&lock.directory, "main.veac", &source).unwrap();

    let result = path::read_target_with(&parent, &source, 64, || {
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&source)
            .unwrap();
        file.write_all(b"change").unwrap();
        file.sync_all().unwrap();
    });
    let Err(error) = result else {
        panic!("same-inode rewrite must invalidate the target snapshot");
    };

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert_eq!(std::fs::read_to_string(source).unwrap(), "change");
}

#[test]
fn target_snapshot_rejects_a_removed_path_after_read() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    let parent = path::resolve(&lock.directory, "main.veac", &source).unwrap();

    let result = path::read_target_with(&parent, &source, 64, || {
        std::fs::remove_file(&source).unwrap();
    });
    let Err(error) = result else {
        panic!("removing the target path must invalidate the snapshot");
    };

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
    assert!(error.to_string().contains("No such file"));
}

#[test]
fn path_identity_rejects_a_removed_target() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "before").unwrap();
    let lock = SourceGraphLock::acquire(temp.path()).unwrap();
    let parent = path::resolve(&lock.directory, "main.veac", &source).unwrap();
    std::fs::remove_file(&source).unwrap();

    let error = path::path_identity(&parent, &source).unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "SOURCE_CHANGED");
}

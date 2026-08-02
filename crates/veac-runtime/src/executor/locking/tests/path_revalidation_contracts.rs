use super::super::{LiveOperations, LOCK_NAME};
use super::acquire;

#[test]
fn missing_output_directory_is_rejected_with_path_context() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing");

    let error = acquire(std::slice::from_ref(&missing)).unwrap_err();
    assert!(error.message.contains("open output directory"));
    assert!(error.message.contains(&missing.display().to_string()));
}

#[test]
fn removed_locked_path_fails_closed_during_revalidation() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output");
    let moved = temp.path().join("moved");
    std::fs::create_dir(&output).unwrap();
    let parent = std::fs::canonicalize(&output).unwrap();
    let locks = acquire(std::slice::from_ref(&parent)).unwrap();
    std::fs::rename(&output, &moved).unwrap();

    let error = locks.directory(&output).unwrap_err();
    assert!(error.message.contains("cannot revalidate output directory"));
}

#[test]
fn replacement_between_canonicalize_and_reopen_fails_closed() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output");
    let moved = temp.path().join("moved");
    std::fs::create_dir(&output).unwrap();
    let canonical = std::fs::canonicalize(&output).unwrap();
    let locks = acquire(std::slice::from_ref(&canonical)).unwrap();

    std::fs::rename(&output, &moved).unwrap();
    let error = locks
        .directory_at_canonical_path(&canonical, &LiveOperations)
        .unwrap_err();
    assert!(error.message.contains("reopen output directory"));
    assert!(error.message.contains(&canonical.display().to_string()));
}

#[cfg(unix)]
#[test]
fn symlink_retarget_to_an_unlocked_directory_fails_closed() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output");
    let moved = temp.path().join("moved");
    let replacement = temp.path().join("replacement");
    std::fs::create_dir(&output).unwrap();
    std::fs::create_dir(&replacement).unwrap();
    let parent = std::fs::canonicalize(&output).unwrap();
    let locks = acquire(std::slice::from_ref(&parent)).unwrap();
    std::fs::rename(&output, moved).unwrap();
    symlink(replacement, &output).unwrap();

    let error = locks.directory(&output).unwrap_err();
    assert!(error.message.contains("changed after lock acquisition"));
}

#[cfg(unix)]
#[test]
fn fifo_lock_file_is_rejected_as_non_regular() {
    let temp = tempfile::tempdir().unwrap();
    let lock_path = temp.path().join(LOCK_NAME);
    let status = std::process::Command::new("mkfifo")
        .arg(&lock_path)
        .status()
        .unwrap();
    assert!(status.success());
    let parent = std::fs::canonicalize(temp.path()).unwrap();

    let error = acquire(std::slice::from_ref(&parent)).unwrap_err();
    assert!(error.message.contains("must be a regular non-symlink file"));
    assert!(lock_path.exists());
}

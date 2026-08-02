use super::{acquire_until, OutputLocks, LOCK_NAME};

mod failure_contracts;
mod path_revalidation_contracts;

#[test]
fn exclusive_directory_lock_rejects_a_concurrent_owner_and_releases_on_drop() {
    let temp = tempfile::tempdir().unwrap();
    let parent = std::fs::canonicalize(temp.path()).unwrap();
    let first = acquire(std::slice::from_ref(&parent)).unwrap();
    let error = acquire(std::slice::from_ref(&parent)).unwrap_err();
    assert!(error.message.contains("locked by another VEAC render"));
    drop(first);
    acquire(&[parent]).unwrap();
}

#[cfg(unix)]
#[test]
fn lock_file_symlink_is_rejected_without_following_it() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let victim = temp.path().join("victim");
    std::fs::write(&victim, b"unchanged").unwrap();
    symlink(&victim, temp.path().join(LOCK_NAME)).unwrap();
    let parent = std::fs::canonicalize(temp.path()).unwrap();
    assert!(acquire(&[parent])
        .unwrap_err()
        .message
        .contains("render lock"));
    assert_eq!(std::fs::read(victim).unwrap(), b"unchanged");
}

#[test]
fn earlier_locks_release_when_a_later_directory_is_busy() {
    let first = tempfile::tempdir().unwrap();
    let second = tempfile::tempdir().unwrap();
    let first = std::fs::canonicalize(first.path()).unwrap();
    let second = std::fs::canonicalize(second.path()).unwrap();
    let busy = acquire(std::slice::from_ref(&second)).unwrap();
    assert!(acquire(&[first.clone(), second.clone()]).is_err());
    acquire(&[first]).unwrap();
    drop(busy);
    acquire(&[second]).unwrap();
}

#[test]
fn locked_directory_rejects_same_path_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output");
    let moved = temp.path().join("moved");
    std::fs::create_dir(&output).unwrap();
    let parent = std::fs::canonicalize(&output).unwrap();
    let locks = acquire(std::slice::from_ref(&parent)).unwrap();
    std::fs::rename(&output, &moved).unwrap();
    std::fs::create_dir(&output).unwrap();
    let error = locks.directory(&output).unwrap_err();
    assert!(error.message.contains("changed after lock acquisition"));
}

#[test]
fn lock_is_exclusive_across_processes() {
    let temp = tempfile::tempdir().unwrap();
    let parent = std::fs::canonicalize(temp.path()).unwrap();
    let ready = temp.path().join("child.ready");
    let release = temp.path().join("child.release");
    let mut child = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "executor::locking::tests::cross_process_lock_child",
            "--nocapture",
        ])
        .env("VEAC_LOCK_PARENT", &parent)
        .env("VEAC_LOCK_READY", &ready)
        .env("VEAC_LOCK_RELEASE", &release)
        .spawn()
        .unwrap();
    wait_for(&ready);
    assert!(acquire(std::slice::from_ref(&parent))
        .unwrap_err()
        .message
        .contains("locked by another VEAC render"));
    std::fs::write(&release, b"release").unwrap();
    assert!(child.wait().unwrap().success());
    acquire(&[parent]).unwrap();
}

#[test]
fn cross_process_lock_child() {
    let Ok(parent) = std::env::var("VEAC_LOCK_PARENT") else {
        return;
    };
    let ready = std::env::var("VEAC_LOCK_READY").unwrap();
    let release = std::env::var("VEAC_LOCK_RELEASE").unwrap();
    let parent = std::path::PathBuf::from(parent);
    let _lock = acquire(std::slice::from_ref(&parent)).unwrap();
    std::fs::write(ready, b"ready").unwrap();
    wait_for(std::path::Path::new(&release));
}

fn wait_for(path: &std::path::Path) {
    for _ in 0..500 {
        if path.exists() {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("timed out waiting for {}", path.display());
}

fn acquire(parents: &[std::path::PathBuf]) -> Result<OutputLocks, crate::RuntimeError> {
    acquire_until(
        parents,
        std::time::Instant::now() + std::time::Duration::from_secs(10),
    )
}

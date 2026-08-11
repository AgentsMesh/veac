#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::*;

#[test]
fn clones_share_one_successful_version_query() {
    let temp = tempfile::tempdir().unwrap();
    let calls = temp.path().join("calls");
    let ffprobe = SystemFfprobe::new(tool(
        temp.path().join("ffprobe"),
        &format!(
            "printf x >> '{}'\nsleep 0.1\nprintf 'ffprobe version shared\\n'\n",
            calls.display()
        ),
    ));
    std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|_| {
                let ffprobe = ffprobe.clone();
                scope.spawn(move || ffprobe.fingerprint().unwrap())
            })
            .collect::<Vec<_>>();
        for handle in handles {
            assert!(handle.join().unwrap().version.contains("shared"));
        }
    });
    assert_eq!(std::fs::read(calls).unwrap(), b"x");
}

#[test]
fn failed_version_initialization_can_retry() {
    let temp = tempfile::tempdir().unwrap();
    let state = temp.path().join("state");
    let calls = temp.path().join("calls");
    let ffprobe = SystemFfprobe::new(tool(
        temp.path().join("ffprobe"),
        &format!(
            "printf x >> '{}'\nif [ ! -e '{}' ]; then touch '{}'; exit 7; fi\nprintf 'ffprobe version retry\\n'\n",
            calls.display(),
            state.display(),
            state.display()
        ),
    ));
    assert!(ffprobe.fingerprint().is_err());
    assert!(ffprobe.fingerprint().unwrap().version.contains("retry"));
    assert_eq!(std::fs::read(calls).unwrap(), b"xx");
}

#[test]
fn clone_waiting_for_version_honors_its_own_deadline() {
    let temp = tempfile::tempdir().unwrap();
    let started = temp.path().join("started");
    let ffprobe = SystemFfprobe::new(tool(
        temp.path().join("ffprobe"),
        &format!(
            "touch '{}'\nsleep 0.2\nprintf 'ffprobe version delayed\\n'\n",
            started.display()
        ),
    ));
    let owner = ffprobe.clone();
    let handle = std::thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_secs(30);
        let pinned = owner.pinned_until(deadline).unwrap();
        owner.version_until(pinned, deadline)
    });
    wait_for(&started);
    let deadline = Instant::now() + Duration::from_millis(20);
    let pinned = ffprobe.pinned_until(deadline).unwrap();
    let error = ffprobe.version_until(pinned, deadline).unwrap_err();
    assert!(matches!(error, ProbeError::ResourceLimit { .. }));
    assert!(handle.join().unwrap().is_ok());
}

#[test]
fn poisoned_version_cache_maps_to_a_stable_tool_error() {
    let ffprobe = SystemFfprobe::new("ffprobe");
    let error = ffprobe.version_cache_error(DeadlineCacheError::Poisoned);
    assert!(matches!(error, ProbeError::ProcessSpawn { .. }));
    assert!(error.to_string().contains("version cache is unavailable"));
}

fn tool(path: PathBuf, body: &str) -> PathBuf {
    std::fs::write(&path, format!("#!/bin/sh\n{body}")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}

fn wait_for(path: &Path) {
    let deadline = Instant::now() + Duration::from_secs(10);
    while !path.exists() {
        assert!(Instant::now() < deadline, "tool did not start");
        std::thread::sleep(Duration::from_millis(2));
    }
}

#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use super::*;

#[test]
fn clones_share_one_successful_fingerprint_query() {
    let temp = tempfile::tempdir().unwrap();
    let calls = temp.path().join("calls");
    let ffmpeg = SystemFfmpeg::new(tool(
        temp.path().join("ffmpeg"),
        &format!(
            "printf x >> '{}'\nsleep 0.1\nprintf 'ffmpeg version shared\\n'\n",
            calls.display()
        ),
    ));
    std::thread::scope(|scope| {
        let handles = (0..8)
            .map(|_| {
                let ffmpeg = ffmpeg.clone();
                scope.spawn(move || ffmpeg.fingerprint().unwrap())
            })
            .collect::<Vec<_>>();
        for handle in handles {
            assert_eq!(handle.join().unwrap().version, "ffmpeg version shared");
        }
    });
    assert_eq!(std::fs::read(calls).unwrap(), b"x");
}

#[test]
fn failed_fingerprint_initialization_can_retry() {
    let temp = tempfile::tempdir().unwrap();
    let state = temp.path().join("state");
    let calls = temp.path().join("calls");
    let ffmpeg = SystemFfmpeg::new(tool(
        temp.path().join("ffmpeg"),
        &format!(
            "printf x >> '{}'\nif [ ! -e '{}' ]; then touch '{}'; exit 7; fi\nprintf 'ffmpeg version retry\\n'\n",
            calls.display(),
            state.display(),
            state.display()
        ),
    ));
    assert!(ffmpeg.fingerprint().is_err());
    assert_eq!(
        ffmpeg.fingerprint().unwrap().version,
        "ffmpeg version retry"
    );
    assert_eq!(std::fs::read(calls).unwrap(), b"xx");
}

#[test]
fn clone_waiting_for_fingerprint_honors_its_own_deadline() {
    let temp = tempfile::tempdir().unwrap();
    let started = temp.path().join("started");
    let ffmpeg = SystemFfmpeg::new(tool(
        temp.path().join("ffmpeg"),
        &format!(
            "touch '{}'\nsleep 0.2\nprintf 'ffmpeg version delayed\\n'\n",
            started.display()
        ),
    ));
    let owner = ffmpeg.clone();
    let handle = std::thread::spawn(move || {
        owner.fingerprint_until(Instant::now() + Duration::from_secs(30))
    });
    wait_for(&started);
    let error = ffmpeg
        .fingerprint_until(Instant::now() + Duration::from_millis(20))
        .unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);
    assert!(handle.join().unwrap().is_ok());
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

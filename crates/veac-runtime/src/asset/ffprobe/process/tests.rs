#![cfg(unix)]

use std::ffi::OsString;
use std::fs;
use std::os::fd::OwnedFd;
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use super::*;

#[test]
fn timeout_kills_ffprobe_descendants_before_they_can_escape() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("descendant-ran");
    let script = temp.path().join("ffprobe-tree.sh");
    fs::write(
        &script,
        format!(
            "#!/bin/sh\n(sleep 0.4; touch '{}') &\nsleep 5\n",
            marker.display()
        ),
    )
    .unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();

    let error = run(
        &script,
        &[],
        1_024,
        Instant::now() + Duration::from_millis(100),
        "descendant test",
    )
    .unwrap_err();
    assert!(matches!(error, ProbeError::ResourceLimit { .. }));
    std::thread::sleep(Duration::from_millis(500));
    assert!(!marker.exists());
}

#[test]
fn bounded_process_captures_both_streams_and_exit_status() {
    let output = shell("printf probe; printf detail >&2; exit 7", 64, deadline()).unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"probe");
    assert_eq!(output.stderr, b"detail");
}

#[test]
fn preexpired_deadline_wins_over_a_missing_executable() {
    let error = run(
        std::path::Path::new("/veac/missing/ffprobe"),
        &[],
        64,
        Instant::now(),
        "preexpired probe",
    )
    .unwrap_err();
    assert!(matches!(error, ProbeError::ResourceLimit { .. }));
}

#[test]
fn missing_executable_preserves_process_spawn_context() {
    let error = run(
        std::path::Path::new("/veac/missing/ffprobe"),
        &[],
        64,
        deadline(),
        "spawn probe",
    )
    .unwrap_err();
    assert!(matches!(error, ProbeError::ProcessSpawn { .. }));
    assert!(error.to_string().contains("/veac/missing/ffprobe"));
}

#[test]
fn stdout_and_stderr_are_independently_bounded() {
    let stdout = shell("printf 123456789", 8, deadline()).unwrap_err();
    assert!(matches!(stdout, ProbeError::ResourceLimit { .. }));

    let bytes = MAX_STDERR_BYTES + 1;
    let script = format!("head -c {bytes} /dev/zero >&2");
    let stderr = shell(&script, 64, deadline()).unwrap_err();
    assert!(matches!(stderr, ProbeError::ResourceLimit { .. }));
}

#[test]
fn tempfile_helpers_distinguish_success_size_rewind_and_read_failures() {
    let binary = std::path::Path::new("ffprobe-test");
    let mut readable = tempfile::tempfile().unwrap();
    use std::io::Write;
    readable.write_all(b"1234").unwrap();
    assert_eq!(length(&readable, binary).unwrap(), 4);
    assert_eq!(read(&mut readable, 4, binary, "stdout").unwrap(), b"1234");

    let mut oversized = tempfile::tempfile().unwrap();
    oversized.write_all(b"12345").unwrap();
    assert!(matches!(
        read(&mut oversized, 4, binary, "stdout"),
        Err(ProbeError::ResourceLimit { .. })
    ));
    let (stream, _peer) = UnixStream::pair().unwrap();
    let mut socket = std::fs::File::from(OwnedFd::from(stream));
    assert!(matches!(
        read(&mut socket, 4, binary, "stdout"),
        Err(ProbeError::ProcessSpawn { .. })
    ));
    let directory = tempfile::tempdir().unwrap();
    let mut directory_file = std::fs::File::open(directory.path()).unwrap();
    assert!(matches!(
        read(&mut directory_file, 4, binary, "stderr"),
        Err(ProbeError::ProcessSpawn { .. })
    ));
}

fn shell(script: &str, stdout: u64, deadline: Instant) -> Result<Output, ProbeError> {
    let arguments = [OsString::from("-c"), OsString::from(script)];
    run(
        std::path::Path::new("/bin/sh"),
        &arguments,
        stdout,
        deadline,
        "bounded probe",
    )
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(3)
}

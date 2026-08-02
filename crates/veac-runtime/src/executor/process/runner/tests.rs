#![cfg(unix)]

use std::path::Path;
use std::time::{Duration, Instant};

use super::*;

#[test]
fn process_limits_bound_stdout_stderr_and_staging_bytes() {
    let output = tempfile::tempdir().unwrap();
    for (script, expected) in [
        ("printf 123456789", "stdout"),
        ("printf 123456789 >&2", "stderr"),
    ] {
        let error = shell(script, Some(output.path()), 8, 8, 64).unwrap_err();
        assert!(error.message.contains(expected), "{}", error.message);
    }
    let path = output.path().join("large");
    let script = format!("printf 123456789 > '{}'", path.display());
    let error = shell(&script, Some(output.path()), 64, 64, 8).unwrap_err();
    assert!(error.message.contains("output exceeded"));
}

#[test]
fn timeout_terminates_the_complete_process_group() {
    let temp = tempfile::tempdir().unwrap();
    let marker = temp.path().join("descendant-finished");
    let script = format!(
        "(sleep 0.15; printf leaked > '{}') & wait",
        marker.display()
    );
    let started = Instant::now();
    let error =
        shell_with_deadline(&script, None, started + Duration::from_millis(40)).unwrap_err();
    assert!(error.message.contains("wall-clock"));
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);
    assert!(started.elapsed() < Duration::from_secs(5));
    std::thread::sleep(Duration::from_millis(250));
    assert!(!marker.exists());
}

#[test]
fn bounded_runner_preserves_success_output_and_rejects_invalid_policy() {
    let output = shell("printf ok; printf detail >&2", None, 64, 64, 64).unwrap();
    assert!(output.status.success());
    assert_eq!(output.stdout, b"ok");
    assert_eq!(output.stderr, b"detail");
    let error = run(
        Path::new("/bin/sh"),
        &["-c".to_owned(), "exit 0".to_owned()],
        ProcessLimits {
            deadline: Instant::now(),
            max_stdout_bytes: 1,
            max_stderr_bytes: 1,
            output_root: None,
            working_directory: None,
            max_output_bytes: 1,
        },
    )
    .unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);
}

fn shell(
    script: &str,
    root: Option<&Path>,
    stdout: u64,
    stderr: u64,
    output: u64,
) -> Result<std::process::Output, crate::RuntimeError> {
    run(
        Path::new("/bin/sh"),
        &["-c".to_owned(), script.to_owned()],
        ProcessLimits {
            deadline: Instant::now() + Duration::from_secs(15),
            max_stdout_bytes: stdout,
            max_stderr_bytes: stderr,
            output_root: root,
            working_directory: None,
            max_output_bytes: output,
        },
    )
}

fn shell_with_deadline(
    script: &str,
    root: Option<&Path>,
    deadline: Instant,
) -> Result<std::process::Output, crate::RuntimeError> {
    run(
        Path::new("/bin/sh"),
        &["-c".to_owned(), script.to_owned()],
        ProcessLimits {
            deadline,
            max_stdout_bytes: 64,
            max_stderr_bytes: 64,
            output_root: root,
            working_directory: None,
            max_output_bytes: 64,
        },
    )
}

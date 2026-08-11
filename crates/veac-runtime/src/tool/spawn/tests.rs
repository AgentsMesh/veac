use std::cell::Cell;
use std::process::Command;
use std::time::{Duration, Instant};

use super::*;

#[test]
fn expired_deadline_stops_before_spawning() {
    let mut command = Command::new("veac-command-that-does-not-exist");
    assert!(matches!(
        spawn_pinned_until(&mut command, Instant::now()),
        Err(PinnedSpawnError::Deadline)
    ));
}

#[test]
fn non_busy_spawn_errors_are_not_retried() {
    let temp = tempfile::tempdir().unwrap();
    let mut command = Command::new(temp.path().join("missing"));
    let error =
        spawn_pinned_until(&mut command, Instant::now() + Duration::from_secs(1)).unwrap_err();
    assert!(
        matches!(error, PinnedSpawnError::Io(error) if error.kind() == std::io::ErrorKind::NotFound)
    );
}

#[test]
fn only_text_busy_is_retryable() {
    let busy = std::io::Error::from_raw_os_error(rustix::io::Errno::TXTBSY.raw_os_error());
    assert!(text_busy(&busy));
    assert!(!text_busy(&std::io::Error::from(
        std::io::ErrorKind::PermissionDenied
    )));
}

#[cfg(target_os = "linux")]
#[test]
fn text_busy_launch_retries_the_same_private_snapshot() {
    let (_temp, writer, mut command) = busy_command();
    let closer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(30));
        drop(writer);
    });
    let mut child =
        spawn_pinned_until(&mut command, Instant::now() + Duration::from_secs(2)).unwrap();
    assert!(child.wait().unwrap().success());
    closer.join().unwrap();
}

#[test]
fn text_busy_launch_stops_at_the_callers_deadline() {
    let start = Instant::now();
    let clock = Cell::new(start);
    let error = retry_until(
        start + Duration::from_millis(3),
        || clock.get(),
        || Err::<(), _>(text_busy_error()),
        |duration| clock.set(clock.get() + duration),
    )
    .unwrap_err();
    assert!(matches!(error, PinnedSpawnError::Deadline));
}

#[test]
fn retry_does_not_launch_after_the_callers_deadline() {
    let start = Instant::now();
    let clock = Cell::new(start);
    let attempts = Cell::new(0);
    let error = retry_until(
        start + RETRY_INTERVAL,
        || clock.get(),
        || {
            attempts.set(attempts.get() + 1);
            (attempts.get() > 1)
                .then_some(())
                .ok_or_else(text_busy_error)
        },
        |duration| clock.set(clock.get() + duration),
    )
    .unwrap_err();
    assert!(matches!(error, PinnedSpawnError::Deadline));
    assert_eq!(attempts.get(), 1);
}

#[test]
fn text_busy_attempt_that_reaches_the_deadline_stops_before_sleeping() {
    let start = Instant::now();
    let deadline = start + Duration::from_millis(3);
    let clock = Cell::new(start);
    let sleeps = Cell::new(0);
    let error = retry_until(
        deadline,
        || clock.get(),
        || {
            clock.set(deadline);
            Err::<(), _>(text_busy_error())
        },
        |_| sleeps.set(sleeps.get() + 1),
    )
    .unwrap_err();
    assert!(matches!(error, PinnedSpawnError::Deadline));
    assert_eq!(sleeps.get(), 0);
}

#[test]
fn text_busy_launch_stops_at_the_retry_window() {
    let start = Instant::now();
    let clock = Cell::new(start);
    let error = retry_until(
        start + Duration::from_secs(2),
        || clock.get(),
        || Err::<(), _>(text_busy_error()),
        |duration| clock.set(clock.get() + duration),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        PinnedSpawnError::Io(error)
            if error.raw_os_error() == Some(rustix::io::Errno::TXTBSY.raw_os_error())
    ));
    assert_eq!(clock.get(), start + RETRY_WINDOW);
}

#[test]
fn text_busy_attempt_that_reaches_the_retry_window_keeps_the_io_error() {
    let start = Instant::now();
    let clock = Cell::new(start);
    let error = retry_until(
        start + Duration::from_secs(2),
        || clock.get(),
        || {
            clock.set(start + RETRY_WINDOW);
            Err::<(), _>(text_busy_error())
        },
        |_| panic!("retry window must stop before sleeping"),
    )
    .unwrap_err();
    assert!(matches!(error, PinnedSpawnError::Io(error) if text_busy(&error)));
}

#[test]
fn text_busy_launch_retries_until_success() {
    let start = Instant::now();
    let clock = Cell::new(start);
    let attempts = Cell::new(0);
    retry_until(
        start + Duration::from_secs(1),
        || clock.get(),
        || {
            attempts.set(attempts.get() + 1);
            if attempts.get() < 3 {
                Err(text_busy_error())
            } else {
                Ok(())
            }
        },
        |duration| clock.set(clock.get() + duration),
    )
    .unwrap();
    assert_eq!(attempts.get(), 3);
    assert_eq!(clock.get(), start + RETRY_INTERVAL * 2);
}

#[cfg(target_os = "linux")]
fn busy_command() -> (tempfile::TempDir, std::fs::File, Command) {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("true");
    std::fs::copy("/bin/true", &executable).unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let writer = std::fs::OpenOptions::new()
        .write(true)
        .open(&executable)
        .unwrap();
    let command = Command::new(executable);
    (temp, writer, command)
}

fn text_busy_error() -> std::io::Error {
    std::io::Error::from_raw_os_error(rustix::io::Errno::TXTBSY.raw_os_error())
}

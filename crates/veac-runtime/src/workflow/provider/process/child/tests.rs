use std::io::{self, Read};
use std::process::Command;
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::*;

struct FailingReader;

impl Read for FailingReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        Err(io::Error::other("read failed"))
    }
}

struct PanickingReader;

impl Read for PanickingReader {
    fn read(&mut self, _: &mut [u8]) -> io::Result<usize> {
        panic!("reader panic")
    }
}

#[test]
fn child_run_round_trips_stdin_stdout_stderr_and_exit_status() {
    let mut command = Command::new("/bin/sh");
    command.args(["-c", "cat; printf detail >&2; exit 7"]);
    let output = run(
        command,
        Some(b"request"),
        ProviderResourceLimits::default(),
        future(),
    )
    .unwrap();
    assert_eq!(output.status.code(), Some(7));
    assert_eq!(output.stdout, b"request");
    assert_eq!(output.stderr, b"detail");
}

#[test]
fn stdin_and_error_helpers_preserve_typed_failure_kinds() {
    stdin(None).unwrap();
    stdin(Some(b"request")).unwrap();
    assert_eq!(
        exceeded::<()>(Stream::Stdout).unwrap_err().kind,
        WorkflowErrorKind::ProtocolViolation
    );
    assert_eq!(
        exceeded::<()>(Stream::Stderr).unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );
    assert_eq!(
        limit::<()>("bounded").unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );
    assert_eq!(internal("pipe").kind, WorkflowErrorKind::Io);
    assert_eq!(
        start_error(io::Error::other("spawn")).kind,
        WorkflowErrorKind::ToolFailure
    );
    assert_eq!(
        thread_error(io::Error::other("thread")).kind,
        WorkflowErrorKind::Io
    );
    assert_eq!(required_pipe(Some(7), "pipe").unwrap(), 7);
    assert_eq!(
        required_pipe::<()>(None, "pipe").unwrap_err().kind,
        WorkflowErrorKind::Io
    );
    assert_eq!(reader_result(Ok(9)).unwrap(), 9);
    assert_eq!(
        reader_result::<()>(Err(io::Error::other("reader")))
            .unwrap_err()
            .kind,
        WorkflowErrorKind::Io
    );
}

#[test]
fn reader_reports_overflow_io_failure_and_panic_without_unwinding() {
    let (sender, receiver) = mpsc::channel();
    let handle = reader::spawn(&b"12345"[..], 4, Stream::Stdout, sender).unwrap();
    assert!(matches!(receiver.recv().unwrap(), Stream::Stdout));
    assert_eq!(reader::finish(handle).unwrap(), b"12345");

    let (sender, _) = mpsc::channel();
    let failed = reader::spawn(FailingReader, 4, Stream::Stderr, sender).unwrap();
    assert_eq!(
        reader::finish(failed).unwrap_err().kind,
        WorkflowErrorKind::Io
    );
    let (sender, _) = mpsc::channel();
    let panicked = reader::spawn(PanickingReader, 4, Stream::Stderr, sender).unwrap();
    assert_eq!(
        reader::finish(panicked).unwrap_err().kind,
        WorkflowErrorKind::Io
    );
}

fn future() -> Instant {
    Instant::now() + Duration::from_secs(2)
}

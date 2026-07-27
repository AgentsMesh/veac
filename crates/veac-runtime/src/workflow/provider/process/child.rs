use std::io::{Seek, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::time::{Duration, Instant};

use super::super::limits::ProviderResourceLimits;
use super::super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

mod reader;

const POLL_INTERVAL: Duration = Duration::from_millis(20);

pub(super) struct ChildOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
}

#[derive(Clone, Copy)]
enum Stream {
    Stdout,
    Stderr,
}

enum End {
    Status(ExitStatus),
    Exceeded(Stream),
    Timeout,
    WaitError(std::io::Error),
}

pub(super) fn run(
    mut command: Command,
    input: Option<&[u8]>,
    limits: ProviderResourceLimits,
    deadline: Instant,
) -> WorkflowResult<ChildOutput> {
    command
        .stdin(stdin(input)?)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    crate::process_group::configure(&mut command);
    let mut child = command.spawn().map_err(start_error)?;
    let stdout = required_pipe(child.stdout.take(), "provider stdout pipe is unavailable")?;
    let stderr = required_pipe(child.stderr.take(), "provider stderr pipe is unavailable")?;
    let (sender, receiver) = mpsc::channel();
    let stdout = match reader_result(reader::spawn(
        stdout,
        limits.max_stdout_bytes,
        Stream::Stdout,
        sender.clone(),
    )) {
        Ok(value) => value,
        Err(error) => {
            crate::process_group::stop(&mut child);
            return Err(error);
        }
    };
    let stderr = match reader_result(reader::spawn(
        stderr,
        limits.max_stderr_bytes,
        Stream::Stderr,
        sender,
    )) {
        Ok(value) => value,
        Err(error) => {
            crate::process_group::stop(&mut child);
            let _ = stdout.join();
            return Err(error);
        }
    };
    let end = loop {
        if let Ok(stream) = receiver.try_recv() {
            crate::process_group::stop(&mut child);
            break End::Exceeded(stream);
        }
        if Instant::now() >= deadline {
            crate::process_group::stop(&mut child);
            break End::Timeout;
        }
        match child.try_wait() {
            Ok(Some(status)) => {
                crate::process_group::terminate(&child);
                break End::Status(status);
            }
            Ok(None) => {}
            Err(error) => {
                crate::process_group::stop(&mut child);
                break End::WaitError(error);
            }
        }
        std::thread::sleep(POLL_INTERVAL);
    };
    let stdout = reader::finish(stdout)?;
    let stderr = reader::finish(stderr)?;
    if stdout.len() as u64 > limits.max_stdout_bytes {
        return exceeded(Stream::Stdout);
    }
    if stderr.len() as u64 > limits.max_stderr_bytes {
        return exceeded(Stream::Stderr);
    }
    match end {
        End::Status(status) => Ok(ChildOutput {
            status,
            stdout,
            stderr,
        }),
        End::Exceeded(stream) => exceeded(stream),
        End::Timeout => limit("provider process exceeded its wall-clock limit"),
        End::WaitError(error) => Err(error.into()),
    }
}

fn stdin(input: Option<&[u8]>) -> WorkflowResult<Stdio> {
    let Some(input) = input else {
        return Ok(Stdio::null());
    };
    let mut file = tempfile::tempfile()?;
    file.write_all(input)?;
    file.rewind()?;
    Ok(Stdio::from(file))
}

fn required_pipe<T>(pipe: Option<T>, message: &str) -> WorkflowResult<T> {
    pipe.ok_or_else(|| internal(message))
}

fn reader_result<T>(result: std::io::Result<T>) -> WorkflowResult<T> {
    result.map_err(thread_error)
}

fn exceeded<T>(stream: Stream) -> WorkflowResult<T> {
    let (kind, message) = match stream {
        Stream::Stdout => (
            WorkflowErrorKind::ProtocolViolation,
            "provider stdout exceeds the protocol byte limit",
        ),
        Stream::Stderr => (
            WorkflowErrorKind::ResourceLimit,
            "provider stderr exceeds the diagnostic byte limit",
        ),
    };
    Err(WorkflowError::new(kind, message))
}

fn limit<T>(message: &str) -> WorkflowResult<T> {
    Err(WorkflowError::new(
        WorkflowErrorKind::ResourceLimit,
        message,
    ))
}

fn internal(message: &str) -> WorkflowError {
    WorkflowError::new(WorkflowErrorKind::Io, message)
}

fn start_error(error: std::io::Error) -> WorkflowError {
    WorkflowError::with_source(
        WorkflowErrorKind::ToolFailure,
        "failed to start provider process",
        error,
    )
}

fn thread_error(error: std::io::Error) -> WorkflowError {
    WorkflowError::with_source(
        WorkflowErrorKind::Io,
        "failed to start provider output reader",
        error,
    )
}

#[cfg(test)]
mod tests;

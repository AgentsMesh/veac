use std::io::Read;
use std::sync::mpsc::Sender;
use std::thread::{self, JoinHandle};

use super::super::super::{WorkflowError, WorkflowErrorKind, WorkflowResult};
use super::Stream;

pub(super) fn spawn(
    source: impl Read + Send + 'static,
    limit: u64,
    stream: Stream,
    sender: Sender<Stream>,
) -> std::io::Result<JoinHandle<std::io::Result<Vec<u8>>>> {
    thread::Builder::new()
        .name("veac-provider-output".to_owned())
        .spawn(move || {
            let mut bytes = Vec::new();
            source.take(limit + 1).read_to_end(&mut bytes)?;
            if bytes.len() as u64 > limit {
                let _ = sender.send(stream);
            }
            Ok(bytes)
        })
}

pub(super) fn finish(handle: JoinHandle<std::io::Result<Vec<u8>>>) -> WorkflowResult<Vec<u8>> {
    match handle.join() {
        Ok(Ok(bytes)) => Ok(bytes),
        Ok(Err(error)) => Err(WorkflowError::with_source(
            WorkflowErrorKind::Io,
            "failed to read bounded provider output",
            error,
        )),
        Err(_) => Err(WorkflowError::new(
            WorkflowErrorKind::Io,
            "provider output reader panicked",
        )),
    }
}

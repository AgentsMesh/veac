use std::io::Write;
use std::time::Instant;

use tempfile::TempDir;
use veac_codegen::emitter::{BackendOutput, BackendTask};

use super::directory::Directory;
use super::{StagedFile, StaleFamily};
use crate::RuntimeError;

const WRITE_CHUNK_BYTES: usize = 64 * 1024;

pub(super) fn stage(
    directory: &TempDir,
    descriptor: &Directory,
    task: &BackendTask,
    content: &[u8],
    deadline: Instant,
) -> Result<(Vec<StagedFile>, Vec<StaleFamily>), RuntimeError> {
    crate::executor::deadline::ensure(deadline)?;
    let BackendOutput::File(target) = &task.output else {
        return Err(RuntimeError::new(
            "write-file task requires one file output",
        ));
    };
    let source = directory.path().join("payload");
    let mut file = descriptor.create_regular("payload")?;
    for chunk in content.chunks(WRITE_CHUNK_BYTES) {
        crate::executor::deadline::ensure(deadline)?;
        file.write_all(chunk).map_err(write_error)?;
        crate::executor::deadline::ensure(deadline)?;
    }
    drop(file);
    crate::executor::deadline::ensure(deadline)?;
    Ok((
        vec![StagedFile {
            source,
            target: target.clone(),
            allow_empty: true,
        }],
        Vec::new(),
    ))
}

fn write_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!("cannot stage output file: {error}"))
}

#[cfg(test)]
#[path = "write/tests.rs"]
mod tests;

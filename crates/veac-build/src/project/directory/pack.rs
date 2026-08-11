use std::fs::{File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};

use super::{relative_text, ENTRY_DIRECTORY, ENTRY_END, ENTRY_FILE, MAGIC, MAX_ENTRIES};
use crate::{CancellationToken, ExecutionError};

struct Entry {
    relative: String,
    path: PathBuf,
    directory: bool,
}

pub(in crate::project) fn pack(
    source: &Path,
    archive: &Path,
    cancellation: &CancellationToken,
) -> Result<(), ExecutionError> {
    let metadata = std::fs::symlink_metadata(source).map_err(io)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(failed(
            "backend directory output must be a non-symlink directory",
        ));
    }
    let root = std::fs::canonicalize(source).map_err(io)?;
    let mut entries = Vec::new();
    collect(&root, &root, &mut entries)?;
    entries.sort_by(|left, right| left.relative.cmp(&right.relative));
    if entries.len() > MAX_ENTRIES {
        return Err(failed("backend directory output exceeds its entry budget"));
    }
    let mut output = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(archive)
        .map_err(io)?;
    output.write_all(MAGIC).map_err(io)?;
    for entry in entries {
        cancelled(cancellation)?;
        output
            .write_all(&[if entry.directory {
                ENTRY_DIRECTORY
            } else {
                ENTRY_FILE
            }])
            .map_err(io)?;
        write_path(&mut output, &entry.relative)?;
        if !entry.directory {
            write_file(&mut output, &entry.path, cancellation)?;
        }
    }
    output.write_all(&[ENTRY_END]).map_err(io)?;
    output.sync_all().map_err(io)
}

fn collect(root: &Path, directory: &Path, entries: &mut Vec<Entry>) -> Result<(), ExecutionError> {
    for result in std::fs::read_dir(directory).map_err(io)? {
        let entry = result.map_err(io)?;
        let path = entry.path();
        let metadata = std::fs::symlink_metadata(&path).map_err(io)?;
        if metadata.file_type().is_symlink() || !(metadata.is_file() || metadata.is_dir()) {
            return Err(failed(
                "backend directory output contains a non-regular entry",
            ));
        }
        entries.push(Entry {
            relative: relative_text(root, &path).map_err(failed)?,
            path: path.clone(),
            directory: metadata.is_dir(),
        });
        if metadata.is_dir() {
            collect(root, &path, entries)?;
        }
    }
    Ok(())
}

fn write_path(output: &mut File, value: &str) -> Result<(), ExecutionError> {
    let length = u32::try_from(value.len()).map_err(|_| failed("directory path is too long"))?;
    output.write_all(&length.to_be_bytes()).map_err(io)?;
    output.write_all(value.as_bytes()).map_err(io)
}

fn write_file(
    output: &mut File,
    path: &Path,
    cancellation: &CancellationToken,
) -> Result<(), ExecutionError> {
    let mut input = File::open(path).map_err(io)?;
    let size = input.metadata().map_err(io)?.len();
    output.write_all(&size.to_be_bytes()).map_err(io)?;
    let digest_offset = output.stream_position().map_err(io)?;
    output.write_all(&[0; 32]).map_err(io)?;
    let mut digest = Sha256::new();
    let mut copied = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        cancelled(cancellation)?;
        let read = input.read(&mut buffer).map_err(io)?;
        if read == 0 {
            break;
        }
        copied = copied
            .checked_add(read as u64)
            .ok_or_else(|| failed("directory file size overflow"))?;
        digest.update(&buffer[..read]);
        output.write_all(&buffer[..read]).map_err(io)?;
    }
    if copied != size {
        return Err(failed("backend directory file changed while archiving"));
    }
    let end = output.stream_position().map_err(io)?;
    output.seek(SeekFrom::Start(digest_offset)).map_err(io)?;
    output.write_all(&digest.finalize()).map_err(io)?;
    output.seek(SeekFrom::Start(end)).map_err(io)?;
    Ok(())
}

fn cancelled(value: &CancellationToken) -> Result<(), ExecutionError> {
    if value.is_cancelled() {
        Err(ExecutionError::cancelled(
            "cancelled while archiving project directory output",
        ))
    } else {
        Ok(())
    }
}

fn io(error: std::io::Error) -> ExecutionError {
    failed(format!("directory archive I/O failed: {error}"))
}

fn failed(message: impl Into<String>) -> ExecutionError {
    ExecutionError::failed(message)
}

#[cfg(test)]
#[path = "pack/coverage_tests.rs"]
mod coverage_tests;

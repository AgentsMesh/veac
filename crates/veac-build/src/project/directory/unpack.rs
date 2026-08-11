use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use sha2::{Digest, Sha256};
use veac_artifact::{ArtifactStore, ContentDigest};

use super::{checked_relative, ENTRY_DIRECTORY, ENTRY_END, ENTRY_FILE, MAGIC, MAX_ENTRIES};
use crate::{BuildError, BuildResult, CancellationToken};

pub(in crate::project) fn publish(
    store: &ArtifactStore,
    artifact: &ContentDigest,
    root: &Path,
    relative: &Path,
    cancellation: &CancellationToken,
) -> BuildResult<PathBuf> {
    let destination = root.join(relative);
    let parent =
        super::super::delivery::checked_parent(root, relative.parent().unwrap_or(Path::new("")))?;
    let verified = store
        .open(artifact)
        .map_err(artifact_error)?
        .ok_or_else(|| BuildError::cache("directory delivery artifact is missing"))?;
    if destination.exists() {
        return if super::existing::matches(
            &destination,
            verified.payload_path(),
            &parent,
            cancellation,
        )? {
            std::fs::canonicalize(destination).map_err(io)
        } else {
            Err(BuildError::invalid(
                "immutable directory delivery already exists with different content",
            ))
        };
    }
    let temporary = tempfile::Builder::new()
        .prefix(".veac-directory-")
        .tempdir_in(&parent)
        .map_err(io)?;
    unpack(verified.payload_path(), temporary.path(), cancellation)?;
    let Some(name) = relative.file_name() else {
        return Err(BuildError::invalid("directory delivery has no file name"));
    };
    let destination = parent.join(name);
    std::fs::rename(temporary.path(), &destination).map_err(io)?;
    OpenOptions::new()
        .read(true)
        .open(&parent)
        .and_then(|file| file.sync_all())
        .map_err(io)?;
    Ok(destination)
}

pub(super) fn unpack(
    archive: &Path,
    destination: &Path,
    cancellation: &CancellationToken,
) -> BuildResult<()> {
    let mut input = File::open(archive).map_err(io)?;
    let mut magic = [0_u8; 8];
    input.read_exact(&mut magic).map_err(io)?;
    if &magic != MAGIC {
        return Err(corrupt("directory artifact has an invalid header"));
    }
    let mut previous = None;
    for _ in 0..=MAX_ENTRIES {
        cancelled(cancellation)?;
        let kind = byte(&mut input)?;
        if kind == ENTRY_END {
            let mut trailing = [0_u8; 1];
            if input.read(&mut trailing).map_err(io)? != 0 {
                return Err(corrupt("directory artifact has trailing bytes"));
            }
            return Ok(());
        }
        if !matches!(kind, ENTRY_DIRECTORY | ENTRY_FILE) {
            return Err(corrupt("directory artifact has an unknown entry kind"));
        }
        let path = read_path(&mut input)?;
        if previous
            .as_deref()
            .is_some_and(|value| value >= path.as_str())
        {
            return Err(corrupt(
                "directory artifact entries are not unique and sorted",
            ));
        }
        let output = destination.join(checked_relative(&path).map_err(corrupt)?);
        previous = Some(path);
        if kind == ENTRY_DIRECTORY {
            std::fs::create_dir(&output).map_err(io)?;
        } else {
            write_file(&mut input, &output, cancellation)?;
        }
    }
    Err(corrupt("directory artifact exceeds its entry budget"))
}

fn read_path(input: &mut File) -> BuildResult<String> {
    let length = u32::from_be_bytes(read_array(input)?) as usize;
    if length == 0 || length > super::MAX_PATH_BYTES {
        return Err(corrupt("directory artifact path length is invalid"));
    }
    let mut bytes = vec![0; length];
    input.read_exact(&mut bytes).map_err(io)?;
    String::from_utf8(bytes).map_err(|_| corrupt("directory artifact path is not UTF-8"))
}

fn write_file(
    input: &mut File,
    output: &Path,
    cancellation: &CancellationToken,
) -> BuildResult<()> {
    let size = u64::from_be_bytes(read_array(input)?);
    let expected: [u8; 32] = read_array(input)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(io)?;
    let mut remaining = size;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    while remaining != 0 {
        cancelled(cancellation)?;
        let wanted = remaining.min(buffer.len() as u64) as usize;
        input.read_exact(&mut buffer[..wanted]).map_err(io)?;
        digest.update(&buffer[..wanted]);
        file.write_all(&buffer[..wanted]).map_err(io)?;
        remaining -= wanted as u64;
    }
    file.sync_all().map_err(io)?;
    let actual: [u8; 32] = digest.finalize().into();
    if actual != expected {
        return Err(corrupt("directory artifact entry digest does not match"));
    }
    Ok(())
}

fn read_array<const N: usize>(input: &mut File) -> BuildResult<[u8; N]> {
    let mut value = [0; N];
    input.read_exact(&mut value).map_err(io)?;
    Ok(value)
}

fn byte(input: &mut File) -> BuildResult<u8> {
    Ok(read_array::<1>(input)?[0])
}

fn cancelled(value: &CancellationToken) -> BuildResult<()> {
    if value.is_cancelled() {
        Err(BuildError::cancelled(
            "cancelled while publishing project directory delivery",
        ))
    } else {
        Ok(())
    }
}

fn artifact_error(error: veac_artifact::ArtifactError) -> BuildError {
    BuildError::cache(format!("directory artifact delivery failure: {error}"))
}

fn corrupt(message: impl Into<String>) -> BuildError {
    BuildError::cache(message)
}

fn io(error: std::io::Error) -> BuildError {
    BuildError::new(
        crate::BuildErrorKind::Cache,
        format!("directory delivery I/O failed: {error}"),
    )
}

#[cfg(test)]
#[path = "unpack/coverage_tests.rs"]
mod coverage_tests;

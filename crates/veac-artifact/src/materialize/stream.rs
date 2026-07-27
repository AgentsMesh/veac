use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos"
))]
use rustix::fs::{fstat, open, FileType, Mode, OFlags};
use sha2::{Digest, Sha256};

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, ContentDigest, DigestAlgorithm};

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos"
))]
pub(super) fn open_regular_while(
    path: &Path,
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<File> {
    super::checked(guard)?;
    let opened = open(
        path,
        OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC,
        Mode::empty(),
    );
    super::checked(guard)?;
    let file = File::from(opened.map_err(open_error)?);
    let state = super::step(guard, || {
        fstat(&file)
            .map_err(std::io::Error::from)
            .map_err(Into::into)
    })?;
    if FileType::from_raw_mode(state.st_mode) != FileType::RegularFile {
        return unsafe_path("verified artifact payload is no longer a regular file");
    }
    Ok(file)
}

#[cfg(not(any(
    target_os = "android",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos"
)))]
pub(super) fn open_regular_while(
    _: &Path,
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<File> {
    super::checked(guard)?;
    unsafe_path("identity-bound materialization is unsupported on this platform")
}

pub(super) fn copy_hash_while(
    input: &mut File,
    output: &mut impl Write,
    expected_size: u64,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<(ContentDigest, u64)> {
    let mut digest = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        super::checked(&mut guard)?;
        let count = input.read(&mut buffer)?;
        super::checked(&mut guard)?;
        if count == 0 {
            break;
        }
        size = size
            .checked_add(count as u64)
            .ok_or_else(super::size_overflow)?;
        if size > expected_size || size > crate::MAX_ARTIFACT_PAYLOAD_BYTES {
            return mismatch("artifact payload grew while it was materialized");
        }
        super::checked(&mut guard)?;
        output.write_all(&buffer[..count])?;
        super::checked(&mut guard)?;
        digest.update(&buffer[..count]);
    }
    Ok((
        ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: format!("{:x}", digest.finalize()),
        },
        size,
    ))
}

pub(super) fn verify_file_while(
    path: &Path,
    expected: &ContentDigest,
    size: u64,
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<()> {
    let metadata = super::step(guard, || {
        std::fs::symlink_metadata(path).map_err(Into::into)
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() != size {
        return mismatch("materialized file size or type differs from the artifact record");
    }
    let mut input = open_regular_while(path, &mut *guard)?;
    let mut sink = std::io::sink();
    let (actual, actual_size) = copy_hash_while(&mut input, &mut sink, size, &mut *guard)?;
    if actual != *expected || actual_size != size {
        return mismatch("materialized file content differs from the artifact record");
    }
    Ok(())
}

#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos"
))]
fn open_error(error: rustix::io::Errno) -> ArtifactError {
    let kind = if error == rustix::io::Errno::LOOP {
        ArtifactErrorKind::UnsafePath
    } else {
        ArtifactErrorKind::Io
    };
    ArtifactError::with_source(
        kind,
        "artifact payload cannot be opened safely",
        std::io::Error::from_raw_os_error(error.raw_os_error()),
    )
}

fn mismatch<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::IdentityMismatch,
        message,
    ))
}

fn unsafe_path<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(ArtifactErrorKind::UnsafePath, message))
}

#[cfg(test)]
#[path = "stream/tests.rs"]
mod tests;

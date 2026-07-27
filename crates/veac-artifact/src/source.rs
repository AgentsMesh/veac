use std::io::Write;
use std::path::Path;

use veac_ir::MediaIdentity;

use crate::{ArtifactResult, OwnedStagedFile};

mod deadline;
mod opened;
mod prefix;
mod reader;

pub use deadline::*;
pub(crate) use opened::verify_open_source_bounded_while;
pub use prefix::*;

use reader::{continue_or_limit, identity_error, read_source_bounded, read_source_bounded_while};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSourceCopy {
    pub identity: MediaIdentity,
    pub size_bytes: u64,
}

pub fn read_verified_source(
    path: &Path,
    expected: Option<&MediaIdentity>,
) -> ArtifactResult<Vec<u8>> {
    read_verified_source_bounded(path, expected, crate::MAX_IN_MEMORY_ARTIFACT_BYTES)
}

pub fn read_verified_source_bounded(
    path: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
) -> ArtifactResult<Vec<u8>> {
    validate_limit(max_bytes, crate::MAX_IN_MEMORY_ARTIFACT_BYTES)?;
    let mut bytes = Vec::new();
    read_source_bounded(
        path,
        expected,
        max_bytes,
        || {},
        |chunk| {
            bytes.extend_from_slice(chunk);
            Ok(())
        },
    )?;
    Ok(bytes)
}

/// Verify a regular source through a fixed-size buffer without retaining its payload.
pub fn verify_source(
    path: &Path,
    expected: Option<&MediaIdentity>,
) -> ArtifactResult<VerifiedSourceCopy> {
    verify_with(path, expected, || {})
}

pub fn verify_source_bounded(
    path: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
) -> ArtifactResult<VerifiedSourceCopy> {
    validate_limit(max_bytes, crate::MAX_VERIFIED_SOURCE_BYTES)?;
    read_source_bounded(path, expected, max_bytes, || {}, |_| Ok(()))
}

pub(crate) fn verify_with(
    path: &Path,
    expected: Option<&MediaIdentity>,
    after_open: impl FnOnce(),
) -> ArtifactResult<VerifiedSourceCopy> {
    read_source_bounded(
        path,
        expected,
        crate::MAX_VERIFIED_SOURCE_BYTES,
        after_open,
        |_| Ok(()),
    )
}

pub fn copy_verified_source(
    source: &Path,
    destination: &Path,
    expected: Option<&MediaIdentity>,
) -> ArtifactResult<VerifiedSourceCopy> {
    copy_with(source, destination, expected, || {})
}

pub fn copy_verified_source_bounded(
    source: &Path,
    destination: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
) -> ArtifactResult<VerifiedSourceCopy> {
    validate_limit(max_bytes, crate::MAX_VERIFIED_SOURCE_BYTES)?;
    copy_with_limit(source, destination, expected, max_bytes, || {})
}

fn validate_limit(value: u64, hard_limit: u64) -> ArtifactResult<()> {
    if value == 0 || value > hard_limit {
        return Err(crate::ArtifactError::new(
            crate::ArtifactErrorKind::InvalidContract,
            "verified source byte limit is outside the supported policy",
        ));
    }
    Ok(())
}

pub(crate) fn copy_with(
    source: &Path,
    destination: &Path,
    expected: Option<&MediaIdentity>,
    after_open: impl FnOnce(),
) -> ArtifactResult<VerifiedSourceCopy> {
    copy_with_limit(
        source,
        destination,
        expected,
        crate::MAX_VERIFIED_SOURCE_BYTES,
        after_open,
    )
}

fn copy_with_limit(
    source: &Path,
    destination: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    after_open: impl FnOnce(),
) -> ArtifactResult<VerifiedSourceCopy> {
    copy_with_limit_while(source, destination, expected, max_bytes, after_open, || {
        true
    })
}

fn copy_with_limit_while(
    source: &Path,
    destination: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    after_open: impl FnOnce(),
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<VerifiedSourceCopy> {
    let parent = destination
        .parent()
        .filter(|value| !value.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."));
    let mut output = OwnedStagedFile::new_in(parent)?;
    let copied = (|| {
        let copied = read_source_bounded_while(
            source,
            expected,
            max_bytes,
            after_open,
            &mut guard,
            |chunk| output.file_mut().write_all(chunk),
        )?;
        continue_or_limit(&mut guard)?;
        output.file_mut().sync_all()?;
        continue_or_limit(&mut guard)?;
        let sealed = output.seal()?;
        if sealed.sha256 != copied.identity.digest || sealed.size_bytes != copied.size_bytes {
            return identity_error("verified snapshot content or size changed after copy");
        }
        Ok(copied)
    })();
    let copied = match copied {
        Ok(value) => value,
        Err(error) => {
            return Err(output.discard_after(error));
        }
    };
    continue_or_limit(&mut guard)?;
    output.persist_noclobber(destination)?;
    Ok(copied)
}

#[cfg(test)]
#[path = "source/tests.rs"]
mod tests;

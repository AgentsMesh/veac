use std::path::Path;

use veac_ir::MediaIdentity;

use super::{copy_with_limit_while, reader, validate_limit, VerifiedSourceCopy};
use crate::ArtifactResult;

pub fn read_verified_source_bounded_while(
    path: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<Vec<u8>> {
    validate_limit(max_bytes, crate::MAX_IN_MEMORY_ARTIFACT_BYTES)?;
    let mut bytes = Vec::new();
    reader::read_source_bounded_while(
        path,
        expected,
        max_bytes,
        || {},
        &mut guard,
        |chunk| {
            bytes.extend_from_slice(chunk);
            Ok(())
        },
    )?;
    Ok(bytes)
}

pub fn verify_source_bounded_while(
    path: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    guard: impl FnMut() -> bool,
) -> ArtifactResult<VerifiedSourceCopy> {
    validate_limit(max_bytes, crate::MAX_VERIFIED_SOURCE_BYTES)?;
    reader::read_source_bounded_while(path, expected, max_bytes, || {}, guard, |_| Ok(()))
}

pub fn copy_verified_source_bounded_while(
    source: &Path,
    destination: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    guard: impl FnMut() -> bool,
) -> ArtifactResult<VerifiedSourceCopy> {
    validate_limit(max_bytes, crate::MAX_VERIFIED_SOURCE_BYTES)?;
    copy_with_limit_while(source, destination, expected, max_bytes, || {}, guard)
}

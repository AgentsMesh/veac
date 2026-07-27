use std::path::Path;

use veac_ir::MediaIdentity;

use super::{reader, validate_limit, VerifiedSourceCopy};
use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

pub const MAX_VERIFIED_SOURCE_PREFIX_BYTES: usize = 4 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedSourcePrefix {
    pub verified: VerifiedSourceCopy,
    pub prefix: Vec<u8>,
}

pub fn verify_source_prefix_bounded_while(
    path: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    prefix_bytes: usize,
    guard: impl FnMut() -> bool,
) -> ArtifactResult<VerifiedSourcePrefix> {
    validate_limit(max_bytes, crate::MAX_VERIFIED_SOURCE_BYTES)?;
    if prefix_bytes > MAX_VERIFIED_SOURCE_PREFIX_BYTES {
        return Err(ArtifactError::new(
            ArtifactErrorKind::InvalidContract,
            "verified source prefix limit exceeds 4096 bytes",
        ));
    }
    let mut prefix = Vec::with_capacity(prefix_bytes);
    let verified = reader::read_source_bounded_while(
        path,
        expected,
        max_bytes,
        || {},
        guard,
        |chunk| {
            let count = prefix_bytes.saturating_sub(prefix.len()).min(chunk.len());
            prefix.extend_from_slice(&chunk[..count]);
            Ok(())
        },
    )?;
    Ok(VerifiedSourcePrefix { verified, prefix })
}

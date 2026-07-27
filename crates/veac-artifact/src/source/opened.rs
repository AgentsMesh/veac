use std::fs::File;

use veac_ir::MediaIdentity;

use super::{reader, validate_limit, VerifiedSourceCopy};
use crate::ArtifactResult;

pub(crate) fn verify_open_source_bounded_while(
    file: &mut File,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    guard: impl FnMut() -> bool,
) -> ArtifactResult<VerifiedSourceCopy> {
    validate_limit(max_bytes, crate::MAX_VERIFIED_SOURCE_BYTES)?;
    reader::read_open_source_bounded_while(file, expected, max_bytes, guard, |_| Ok(()))
}

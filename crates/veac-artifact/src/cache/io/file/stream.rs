use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

use sha2::{Digest, Sha256};

use super::{regular_source, require_payload_limit};
use crate::{ArtifactResult, ContentDigest, DigestAlgorithm};

pub(super) fn fingerprint_source_while(
    path: &Path,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<(ContentDigest, u64)> {
    checked(&mut guard)?;
    let mut input = regular_source(path)?;
    checked(&mut guard)?;
    hash_while(&mut input, None, guard)
}

pub(super) fn copy_hashed_while(
    input: &mut File,
    output: &mut File,
    guard: impl FnMut() -> bool,
) -> ArtifactResult<(ContentDigest, u64)> {
    hash_while(input, Some(output), guard)
}

fn hash_while(
    input: &mut File,
    mut output: Option<&mut File>,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<(ContentDigest, u64)> {
    let mut digest = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        checked(&mut guard)?;
        let count = input.read(&mut buffer)?;
        checked(&mut guard)?;
        if count == 0 {
            break;
        }
        size = size.checked_add(count as u64).ok_or_else(|| {
            super::resource_limit::<()>("artifact payload size overflow").unwrap_err()
        })?;
        require_payload_limit(size)?;
        if let Some(output) = output.as_mut() {
            checked(&mut guard)?;
            output.write_all(&buffer[..count])?;
            checked(&mut guard)?;
        }
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

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    crate::cache::guard::check(guard)
}

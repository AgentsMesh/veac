use std::fs::File;
use std::io::{Read, Seek};

use rustix::fs::{fstat, Stat};
use sha2::{Digest, Sha256};
use veac_ir::{HashAlgorithm, MediaIdentity};

use super::file::{declared_size, io_error, same_file_state};
use super::{continue_or_limit, identity_error, limit_error};
use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, VerifiedSourceCopy};

pub(super) fn read_open_source_bounded_guarded(
    file: &mut File,
    before: &Stat,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    before_confirmation: impl FnOnce(),
    mut guard: impl FnMut() -> bool,
    mut consume: impl FnMut(&[u8]) -> std::io::Result<()>,
) -> ArtifactResult<VerifiedSourceCopy> {
    continue_or_limit(&mut guard)?;
    let declared_size = declared_size(before)?;
    let (digest, size_bytes) = hash_open_file(file, max_bytes, &mut guard, &mut consume)?;
    continue_or_limit(&mut guard)?;
    let after = fstat(&*file).map_err(io_error)?;
    if !same_file_state(before, &after) || declared_size != size_bytes {
        return identity_error("source changed while creating a verified snapshot");
    }
    let identity = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest,
    };
    if expected.is_some_and(|value| value != &identity) {
        return identity_error("source does not match its pinned SHA-256 identity");
    }
    if expected.is_none() {
        before_confirmation();
        continue_or_limit(&mut guard)?;
        file.rewind()?;
        if hash_open_file(file, max_bytes, &mut guard, |_| Ok(()))?
            != (identity.digest.clone(), size_bytes)
        {
            return identity_error("source changed while confirming its snapshot identity");
        }
    }
    Ok(VerifiedSourceCopy {
        identity,
        size_bytes,
    })
}

fn hash_open_file(
    file: &mut File,
    max_bytes: u64,
    mut guard: impl FnMut() -> bool,
    mut consume: impl FnMut(&[u8]) -> std::io::Result<()>,
) -> ArtifactResult<(String, u64)> {
    let mut digest = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        continue_or_limit(&mut guard)?;
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        size = checked_size(size, count)?;
        if size > max_bytes {
            return limit_error(max_bytes);
        }
        digest.update(&buffer[..count]);
        consume(&buffer[..count])?;
        continue_or_limit(&mut guard)?;
    }
    Ok((hex(digest.finalize()), size))
}

pub(super) fn checked_size(size: u64, count: usize) -> ArtifactResult<u64> {
    size.checked_add(count as u64).ok_or_else(|| {
        ArtifactError::new(ArtifactErrorKind::InvalidContract, "source size overflow")
    })
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        write!(&mut output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}

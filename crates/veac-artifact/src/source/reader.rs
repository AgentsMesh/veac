use std::fs::File;
use std::io::Seek;
use std::path::Path;

use rustix::fs::fstat;
use veac_ir::MediaIdentity;

use super::VerifiedSourceCopy;
use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult};

mod file;
mod opened;

use file::{
    declared_size, io_error, open_regular, require_regular, same_file_state, validate_expected,
};
#[cfg(test)]
use opened::checked_size;
use opened::read_open_source_bounded_guarded;

struct ReadHooks<AfterOpen, BeforeConfirmation, BeforePathConfirmation> {
    after_open: AfterOpen,
    before_confirmation: BeforeConfirmation,
    before_path_confirmation: BeforePathConfirmation,
}

pub(super) fn read_source_bounded(
    path: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    after_open: impl FnOnce(),
    consume: impl FnMut(&[u8]) -> std::io::Result<()>,
) -> ArtifactResult<VerifiedSourceCopy> {
    read_source_bounded_while(path, expected, max_bytes, after_open, || true, consume)
}

pub(super) fn read_source_bounded_while(
    path: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    after_open: impl FnOnce(),
    guard: impl FnMut() -> bool,
    consume: impl FnMut(&[u8]) -> std::io::Result<()>,
) -> ArtifactResult<VerifiedSourceCopy> {
    read_source_bounded_guarded(
        path,
        expected,
        max_bytes,
        ReadHooks {
            after_open,
            before_confirmation: || {},
            before_path_confirmation: || {},
        },
        guard,
        consume,
    )
}

fn read_source_bounded_guarded(
    path: &Path,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    hooks: ReadHooks<impl FnOnce(), impl FnOnce(), impl FnOnce()>,
    mut guard: impl FnMut() -> bool,
    consume: impl FnMut(&[u8]) -> std::io::Result<()>,
) -> ArtifactResult<VerifiedSourceCopy> {
    let ReadHooks {
        after_open,
        before_confirmation,
        before_path_confirmation,
    } = hooks;
    continue_or_limit(&mut guard)?;
    validate_expected(expected)?;
    let (mut file, before) = open_regular(path)?;
    let declared_size = declared_size(&before)?;
    if declared_size > max_bytes {
        return limit_error(max_bytes);
    }
    after_open();
    let verified = read_open_source_bounded_guarded(
        &mut file,
        &before,
        expected,
        max_bytes,
        before_confirmation,
        &mut guard,
        consume,
    )?;
    before_path_confirmation();
    continue_or_limit(&mut guard)?;
    let (_, current) = open_regular(path)?;
    continue_or_limit(&mut guard)?;
    if !same_file_state(&before, &current) {
        return identity_error("source path changed while confirming its snapshot authority");
    }
    Ok(verified)
}

pub(super) fn read_open_source_bounded_while(
    file: &mut File,
    expected: Option<&MediaIdentity>,
    max_bytes: u64,
    mut guard: impl FnMut() -> bool,
    consume: impl FnMut(&[u8]) -> std::io::Result<()>,
) -> ArtifactResult<VerifiedSourceCopy> {
    continue_or_limit(&mut guard)?;
    validate_expected(expected)?;
    let before = fstat(&*file).map_err(io_error)?;
    require_regular(&before)?;
    if declared_size(&before)? > max_bytes {
        return limit_error(max_bytes);
    }
    file.rewind()?;
    read_open_source_bounded_guarded(file, &before, expected, max_bytes, || {}, guard, consume)
}

fn limit_error<T>(max_bytes: u64) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::ResourceLimit,
        format!("verified source exceeds {max_bytes} byte read limit"),
    ))
}

pub(super) fn continue_or_limit(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    if guard() {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::ResourceLimit,
            "verified source exceeded its caller deadline",
        ))
    }
}

pub(super) fn identity_error<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::IdentityMismatch,
        message,
    ))
}

#[cfg(test)]
mod tests;

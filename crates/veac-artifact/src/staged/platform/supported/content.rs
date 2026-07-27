use std::fs::File;
use std::os::unix::fs::FileExt;

use rustix::fs::{fstat, FileType, Stat};
use sha2::{Digest, Sha256};

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, StagedContent};

pub(super) fn fingerprint_while(
    file: &File,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<StagedContent> {
    checked(&mut guard)?;
    let before = state(file)?;
    checked(&mut guard)?;
    require_owned_regular(&before)?;
    let declared_size = u64::try_from(before.st_size).map_err(|_| {
        ArtifactError::new(
            ArtifactErrorKind::InvalidContract,
            "staged payload size is negative",
        )
    })?;
    let mut digest = Sha256::new();
    let mut offset = 0_u64;
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        checked(&mut guard)?;
        let count = file.read_at(&mut buffer, offset)?;
        checked(&mut guard)?;
        if count == 0 {
            break;
        }
        offset = offset.checked_add(count as u64).ok_or_else(|| {
            ArtifactError::new(
                ArtifactErrorKind::InvalidContract,
                "staged payload size overflow",
            )
        })?;
        digest.update(&buffer[..count]);
    }
    checked(&mut guard)?;
    let after = state(file)?;
    checked(&mut guard)?;
    require_owned_regular(&after)?;
    if !same_state(&before, &after) || declared_size != offset {
        return Err(ArtifactError::new(
            ArtifactErrorKind::IdentityMismatch,
            "staged payload changed while its content identity was computed",
        ));
    }
    Ok(StagedContent {
        sha256: hex(digest.finalize()),
        size_bytes: offset,
    })
}

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    crate::staged::check_guard(guard)
}

fn state(file: &File) -> ArtifactResult<Stat> {
    fstat(file).map_err(|error| {
        ArtifactError::with_source(
            ArtifactErrorKind::Io,
            "staged output cannot inspect payload content",
            std::io::Error::from_raw_os_error(error.raw_os_error()),
        )
    })
}

fn require_owned_regular(state: &Stat) -> ArtifactResult<()> {
    if FileType::from_raw_mode(state.st_mode) != FileType::RegularFile || state.st_nlink != 1 {
        return Err(ArtifactError::new(
            ArtifactErrorKind::UnsafePath,
            "staged payload must be a regular file with exactly one link",
        ));
    }
    Ok(())
}

fn same_state(left: &Stat, right: &Stat) -> bool {
    left.st_dev == right.st_dev
        && left.st_ino == right.st_ino
        && left.st_nlink == right.st_nlink
        && left.st_size == right.st_size
        && left.st_mtime == right.st_mtime
        && left.st_mtime_nsec == right.st_mtime_nsec
        && left.st_ctime == right.st_ctime
        && left.st_ctime_nsec == right.st_ctime_nsec
}

fn hex(bytes: impl AsRef<[u8]>) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.as_ref().len() * 2);
    for byte in bytes.as_ref() {
        write!(&mut output, "{byte:02x}").expect("writing to a string cannot fail");
    }
    output
}

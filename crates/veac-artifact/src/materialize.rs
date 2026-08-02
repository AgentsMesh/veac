use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{ArtifactError, ArtifactErrorKind, ArtifactResult, OwnedStagedFile, VerifiedArtifact};

mod destination;
mod stream;

pub fn materialize(artifact: &VerifiedArtifact, destination: &Path) -> ArtifactResult<PathBuf> {
    materialize_while(artifact, destination, || true)
}

pub fn materialize_while(
    artifact: &VerifiedArtifact,
    destination: &Path,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<PathBuf> {
    checked(&mut guard)?;
    let (parent, name) = destination::checked(destination, &mut guard)?;
    let destination = parent.join(name);
    if destination::existing(&destination, &mut guard)? {
        stream::verify_file_while(
            &destination,
            &artifact.record().content,
            artifact.record().size_bytes,
            &mut guard,
        )?;
        return canonical(&destination, &mut guard, false);
    }
    copy_atomic_while(artifact, &destination, &parent, &mut guard)?;
    canonical(&destination, &mut guard, true)
}

fn copy_atomic_while(
    artifact: &VerifiedArtifact,
    destination: &Path,
    parent: &Path,
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<()> {
    let mut source = stream::open_regular_while(artifact.payload_path(), &mut *guard)?;
    let mut staged = step(guard, || OwnedStagedFile::new_in(parent))?;
    let result = (|| {
        let (content, size) = stream::copy_hash_while(
            &mut source,
            staged.file_mut(),
            artifact.record().size_bytes,
            &mut *guard,
        )?;
        step(guard, || staged.file_mut().flush().map_err(Into::into))?;
        step(guard, || staged.file_mut().sync_all().map_err(Into::into))?;
        if content != artifact.record().content || size != artifact.record().size_bytes {
            return mismatch("artifact payload changed before materialization completed");
        }
        let sealed = staged.seal_while(&mut *guard)?;
        if sealed.sha256 != content.value || sealed.size_bytes != size {
            return mismatch("staged materialization differs from the copied artifact");
        }
        if destination::existing(destination, &mut *guard)? {
            return mismatch("materialization destination appeared during staging");
        }
        Ok(())
    })();
    if let Err(error) = result {
        return Err(staged.discard_after(error));
    }
    staged.persist_noclobber_while(destination, &mut *guard)?;
    let directory =
        step(guard, || File::open(parent).map_err(Into::into)).map_err(ArtifactError::committed)?;
    step(guard, || directory.sync_all().map_err(Into::into)).map_err(ArtifactError::committed)
}

fn canonical(
    path: &Path,
    guard: &mut impl FnMut() -> bool,
    committed: bool,
) -> ArtifactResult<PathBuf> {
    let result = step(guard, || std::fs::canonicalize(path).map_err(Into::into));
    if committed {
        result.map_err(ArtifactError::committed)
    } else {
        result
    }
}

fn step<T>(
    guard: &mut impl FnMut() -> bool,
    operation: impl FnOnce() -> ArtifactResult<T>,
) -> ArtifactResult<T> {
    checked(guard)?;
    let value = operation()?;
    checked(guard)?;
    Ok(value)
}

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    if guard() {
        Ok(())
    } else {
        Err(ArtifactError::new(
            ArtifactErrorKind::ResourceLimit,
            "artifact materialization exceeded its caller resource guard",
        ))
    }
}

fn mismatch<T>(message: &str) -> ArtifactResult<T> {
    Err(ArtifactError::new(
        ArtifactErrorKind::IdentityMismatch,
        message,
    ))
}

pub(crate) fn size_overflow() -> ArtifactError {
    ArtifactError::new(ArtifactErrorKind::InvalidContract, "artifact size overflow")
}

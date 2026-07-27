use std::ffi::OsString;

use super::{
    BoundDirectory, BoundFile, CacheStage, DirectoryLock, StageOpen, DESCRIPTOR, PAYLOAD, RECORD,
};
use crate::{ArtifactError, ArtifactResult};

pub(super) fn begin(
    parent: super::super::authority::BoundChain,
    target: OsString,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<StageOpen> {
    let prefix_lock = DirectoryLock::exclusive_while(parent.current(), &mut guard)?;
    parent.verify()?;
    let directory = match BoundDirectory::open(parent.try_clone()?, target.clone())? {
        Some(directory) => {
            if super::super::state::marker(&directory)?.is_some() {
                return Ok(StageOpen::Conflict);
            }
            let (parent, target) = super::super::recovery::remove_incomplete(directory)?;
            BoundDirectory::create(parent, target)?
        }
        None => BoundDirectory::create(parent, target)?,
    };
    create_entries(directory, prefix_lock)
}

fn create_entries(
    directory: BoundDirectory,
    prefix_lock: DirectoryLock,
) -> ArtifactResult<StageOpen> {
    let descriptor = BoundFile::create(directory.file(), DESCRIPTOR.as_ref())
        .map_err(|primary| cleanup_after(&directory, &[], primary))?;
    let payload = BoundFile::create(directory.file(), PAYLOAD.as_ref())
        .map_err(|primary| cleanup_after(&directory, &[&descriptor], primary))?;
    let record = BoundFile::create(directory.file(), RECORD.as_ref())
        .map_err(|primary| cleanup_after(&directory, &[&descriptor, &payload], primary))?;
    Ok(StageOpen::Ready(Box::new(CacheStage {
        directory,
        descriptor,
        payload,
        record,
        sealed: false,
        _prefix_lock: prefix_lock,
    })))
}

fn cleanup_after(
    directory: &BoundDirectory,
    entries: &[&BoundFile],
    primary: ArtifactError,
) -> ArtifactError {
    let cleanup = (|| {
        directory.verify()?;
        for entry in entries {
            entry.unlink(directory.file())?;
        }
        directory.remove_empty()?;
        directory.sync_parent()
    })();
    match cleanup {
        Ok(()) => primary,
        Err(error) => primary.with_cleanup_failure(error, "cache setup cleanup failed"),
    }
}

#[cfg(test)]
#[path = "create/tests.rs"]
mod tests;

use std::path::Path;

use crate::{ArtifactError, ArtifactResult};

use super::super::authority::target_parent;
use super::super::directory::BoundDirectory;
use super::super::entry::BoundFile;
use super::super::lock::DirectoryLock;
use super::super::{recovery, state};

pub(super) fn remove_while_with(
    root: &Path,
    directory: &Path,
    before_remove: impl FnOnce(&Path),
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<bool> {
    checked(&mut guard)?;
    let parent = target_parent(root, directory, false)?;
    checked(&mut guard)?;
    let Some((parent, name)) = parent else {
        return Ok(false);
    };
    let _lock = DirectoryLock::exclusive_while(parent.current(), &mut guard)?;
    parent.verify()?;
    let Some(bound) = BoundDirectory::open(parent, name)? else {
        return Ok(false);
    };
    checked(&mut guard)?;
    let Some(marker) = state::marker(&bound)? else {
        recovery::remove_incomplete(bound)?;
        return Ok(false);
    };
    let descriptor = open(&bound, state::DESCRIPTOR, &mut guard)?;
    let payload = open(&bound, state::PAYLOAD, &mut guard)?;
    let record = open(&bound, state::RECORD, &mut guard)?;
    before_remove(bound.path());
    checked(&mut guard)?;
    verify(
        &bound,
        [&marker, &descriptor, &payload, &record],
        &mut guard,
    )?;
    checked(&mut guard)?;
    marker.unlink(bound.file())?;
    let mut expired = checked(&mut guard).err();
    cleanup_step(&mut guard, &mut expired, || bound.verify())?;
    cleanup_step(&mut guard, &mut expired, || descriptor.unlink(bound.file()))?;
    cleanup_step(&mut guard, &mut expired, || bound.verify())?;
    cleanup_step(&mut guard, &mut expired, || payload.unlink(bound.file()))?;
    cleanup_step(&mut guard, &mut expired, || bound.verify())?;
    cleanup_step(&mut guard, &mut expired, || record.unlink(bound.file()))?;
    cleanup_step(&mut guard, &mut expired, || bound.verify())?;
    cleanup_step(&mut guard, &mut expired, || bound.remove_empty())?;
    cleanup_step(&mut guard, &mut expired, || bound.sync_parent())?;
    if let Some(error) = expired {
        return Err(error.committed());
    }
    Ok(true)
}

fn open(
    directory: &BoundDirectory,
    name: &str,
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<BoundFile> {
    checked(guard)?;
    let file = BoundFile::open(directory.file(), name.as_ref())?;
    checked(guard)?;
    Ok(file)
}

fn verify(
    directory: &BoundDirectory,
    files: [&BoundFile; 4],
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<()> {
    directory.verify()?;
    for file in files {
        file.verify_while(directory.file(), &mut *guard)?;
    }
    Ok(())
}

fn cleanup_step<T>(
    guard: &mut impl FnMut() -> bool,
    expired: &mut Option<ArtifactError>,
    operation: impl FnOnce() -> ArtifactResult<T>,
) -> ArtifactResult<T> {
    observe(guard, expired);
    let value = operation().map_err(ArtifactError::committed)?;
    observe(guard, expired);
    Ok(value)
}

fn observe(guard: &mut impl FnMut() -> bool, expired: &mut Option<ArtifactError>) {
    if expired.is_none() {
        *expired = checked(guard).err();
    }
}

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    crate::cache::guard::check(guard)
}

use std::fs::File;
use std::path::Path;

use rustix::fs::{fstat, open, FileType, Mode, OFlags};

use super::authority::target_parent;
use super::common::{corrupt, resource_limit, source_open_error};
use super::operations::{metadata_bytes, require_payload_limit};
use super::stage::{CacheStage, ExpectedContent, StageOpen};
use super::PublishOutcome;
use crate::{
    ArtifactDescriptor, ArtifactError, ArtifactErrorKind, ArtifactRecord, ArtifactResult,
    ContentDigest,
};

mod stream;

pub(in crate::cache) fn write_file_atomic_while(
    root: &Path,
    directory: &Path,
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
    source: &Path,
    guard: impl FnMut() -> bool,
) -> ArtifactResult<PublishOutcome> {
    write_file_atomic_while_with(
        root,
        directory,
        descriptor,
        record,
        source,
        |_| {},
        |_| {},
        |_| {},
        guard,
    )
}

pub(in crate::cache) fn fingerprint_source_while(
    path: &Path,
    guard: impl FnMut() -> bool,
) -> ArtifactResult<(ContentDigest, u64)> {
    stream::fingerprint_source_while(path, guard)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn write_file_atomic_while_with(
    root: &Path,
    directory: &Path,
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
    source: &Path,
    after_chain_open: impl FnOnce(&Path),
    after_stage_create: impl FnOnce(&Path),
    before_publish: impl FnOnce(&Path),
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<PublishOutcome> {
    checked(&mut guard)?;
    require_payload_limit(record.size_bytes)?;
    let (descriptor_bytes, record_bytes) = metadata_bytes(descriptor, record)?;
    checked(&mut guard)?;
    let parent = target_parent(root, directory, true)?;
    checked(&mut guard)?;
    let Some((parent, target)) = parent else {
        return corrupt("cache parent disappeared while being created");
    };
    after_chain_open(parent.path());
    checked(&mut guard)?;
    before_publish(directory);
    checked(&mut guard)?;
    let mut stage = match CacheStage::begin(parent, target, &mut guard)? {
        StageOpen::Ready(stage) => *stage,
        StageOpen::Conflict => return Ok(PublishOutcome::Conflict),
    };
    after_stage_create(stage.path());
    checked(&mut guard)?;
    stage.write_metadata(&descriptor_bytes, &record_bytes)?;
    checked(&mut guard)?;
    let mut input = regular_source(source)?;
    checked(&mut guard)?;
    let (content, size) = stream::copy_hashed_while(&mut input, stage.payload(), &mut guard)?;
    if content != record.content || size != record.size_bytes {
        return Err(ArtifactError::new(
            ArtifactErrorKind::IdentityMismatch,
            "artifact source changed while being stored",
        ));
    }
    stage.seal_while(
        &ExpectedContent {
            descriptor: &descriptor_bytes,
            record: &record_bytes,
            payload_digest: &record.content,
            payload_size: record.size_bytes,
        },
        guard,
    )?;
    Ok(PublishOutcome::Published)
}

pub(super) fn regular_source(path: &Path) -> ArtifactResult<File> {
    let flags = OFlags::RDONLY | OFlags::NOFOLLOW | OFlags::NONBLOCK | OFlags::CLOEXEC;
    let file = File::from(open(path, flags, Mode::empty()).map_err(source_open_error)?);
    let stat = fstat(&file).map_err(std::io::Error::from)?;
    if FileType::from_raw_mode(stat.st_mode) != FileType::RegularFile {
        return Err(ArtifactError::new(
            ArtifactErrorKind::UnsafePath,
            "artifact source must be a regular non-symlink file",
        ));
    }
    require_payload_limit(stat.st_size as u64)?;
    Ok(file)
}

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    crate::cache::guard::check(guard)
}

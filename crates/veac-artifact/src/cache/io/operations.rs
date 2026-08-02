use std::path::Path;

use super::common::{resource_limit, serialization_error};
use super::PublishOutcome;
use crate::{
    canonical_descriptor_bytes, ArtifactDescriptor, ArtifactRecord, ArtifactResult,
    MAX_ARTIFACT_METADATA_BYTES, MAX_ARTIFACT_PAYLOAD_BYTES,
};

mod guarded;
mod removal;

pub(in crate::cache) fn read_while(
    root: &Path,
    directory: &Path,
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<Option<(ArtifactDescriptor, ArtifactRecord, Vec<u8>)>> {
    crate::cache::guard::check(&mut guard)?;
    let Some(mut inspected) = super::inspect::inspect_bounded_while(
        root,
        directory,
        MAX_ARTIFACT_METADATA_BYTES,
        &mut guard,
    )?
    else {
        return Ok(None);
    };
    let payload = inspected.read_payload_while(&mut guard)?;
    crate::cache::guard::check(&mut guard)?;
    Ok(Some((inspected.descriptor, inspected.record, payload)))
}

#[allow(clippy::too_many_arguments)]
pub(in crate::cache) fn write_atomic_while_with(
    root: &Path,
    directory: &Path,
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
    payload: &[u8],
    after_chain_open: impl FnOnce(&Path),
    after_stage_create: impl FnOnce(&Path),
    before_publish: impl FnOnce(&Path),
    guard: impl FnMut() -> bool,
) -> ArtifactResult<PublishOutcome> {
    guarded::write_atomic_guarded_with(
        root,
        directory,
        descriptor,
        record,
        payload,
        after_chain_open,
        after_stage_create,
        before_publish,
        guard,
    )
}

pub(in crate::cache) fn remove_while(
    root: &Path,
    directory: &Path,
    guard: impl FnMut() -> bool,
) -> ArtifactResult<bool> {
    remove_while_with(root, directory, |_| {}, guard)
}

pub(super) fn remove_while_with(
    root: &Path,
    directory: &Path,
    before_remove: impl FnOnce(&Path),
    guard: impl FnMut() -> bool,
) -> ArtifactResult<bool> {
    removal::remove_while_with(root, directory, before_remove, guard)
}

pub(super) fn metadata_bytes(
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
) -> ArtifactResult<(Vec<u8>, Vec<u8>)> {
    let descriptor = canonical_descriptor_bytes(descriptor)?;
    let record = serde_json_canonicalizer::to_vec(record).map_err(serialization_error)?;
    if descriptor.len().saturating_add(record.len()) as u64 > MAX_ARTIFACT_METADATA_BYTES {
        return resource_limit("artifact metadata exceeds the cache limit");
    }
    Ok((descriptor, record))
}

pub(super) fn require_payload_limit(size: u64) -> ArtifactResult<()> {
    if size > MAX_ARTIFACT_PAYLOAD_BYTES {
        return resource_limit("artifact payload exceeds the cache limit");
    }
    Ok(())
}

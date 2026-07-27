use std::io::Write;

use super::*;
use crate::cache::io::authority::target_parent;
use crate::cache::io::common::corrupt;
use crate::cache::io::stage::{CacheStage, ExpectedContent, StageOpen};

#[allow(clippy::too_many_arguments)]
pub(super) fn write_atomic_guarded_with(
    root: &Path,
    directory: &Path,
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
    payload: &[u8],
    after_chain_open: impl FnOnce(&Path),
    after_stage_create: impl FnOnce(&Path),
    before_publish: impl FnOnce(&Path),
    mut guard: impl FnMut() -> bool,
) -> ArtifactResult<PublishOutcome> {
    checked(&mut guard)?;
    require_payload_limit(payload.len() as u64)?;
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
    for chunk in payload.chunks(64 * 1024) {
        checked(&mut guard)?;
        stage.payload().write_all(chunk)?;
        checked(&mut guard)?;
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

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    crate::cache::guard::check(guard)
}

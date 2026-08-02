use std::path::Path;

use super::*;

pub(super) fn inspect_bounded_with(
    root: &Path,
    directory: &Path,
    metadata_budget: u64,
    after_entry_open: impl FnOnce(&Path),
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<Option<InspectedArtifact>> {
    checked(guard)?;
    let parent = target_parent(root, directory, false)?;
    checked(guard)?;
    let Some((parent, name)) = parent else {
        return Ok(None);
    };
    let prefix_lock = DirectoryLock::shared_while(parent.current(), &mut *guard)?;
    parent.verify()?;
    let bound = BoundDirectory::open(parent, name)?;
    checked(guard)?;
    let Some(bound) = bound else {
        return Ok(None);
    };
    let Some(marker) = state::marker(&bound)? else {
        return Ok(None);
    };
    checked(guard)?;
    let mut descriptor_file = open(&bound, state::DESCRIPTOR, guard)?;
    let payload = open(&bound, state::PAYLOAD, guard)?;
    let mut record_file = open(&bound, state::RECORD, guard)?;
    after_entry_open(bound.path());
    checked(guard)?;
    let descriptor_size = descriptor_file.size()?;
    let record_size = record_file.size()?;
    let metadata_bytes = descriptor_size.saturating_add(record_size);
    if metadata_bytes > MAX_ARTIFACT_METADATA_BYTES || metadata_bytes > metadata_budget {
        return super::super::common::resource_limit(
            "cached artifact metadata exceeds the size limit",
        );
    }
    let descriptor_bytes = descriptor_file.read_bounded_while(
        MAX_ARTIFACT_METADATA_BYTES,
        "cached artifact descriptor exceeds the metadata limit",
        &mut *guard,
    )?;
    checked(guard)?;
    let descriptor: ArtifactDescriptor = decode(&descriptor_bytes)?;
    checked(guard)?;
    if descriptor_bytes != canonical_descriptor_bytes(&descriptor).map_err(metadata_error)? {
        return corrupt("cached artifact descriptor is not canonical JSON");
    }
    let record_bytes = record_file.read_bounded_while(
        MAX_ARTIFACT_METADATA_BYTES - descriptor_size,
        "cached artifact record exceeds the metadata limit",
        &mut *guard,
    )?;
    checked(guard)?;
    let record: ArtifactRecord = decode(&record_bytes)?;
    validate_record(&record, &record_bytes)?;
    checked(guard)?;
    if payload.size()? != record.size_bytes {
        return corrupt("cached artifact payload size does not match its record");
    }
    let inspected = InspectedArtifact {
        descriptor,
        record,
        payload,
        descriptor_file,
        record_file,
        marker,
        directory: bound,
        metadata_bytes,
        _prefix_lock: prefix_lock,
    };
    verify_entries(&inspected, guard)?;
    Ok(Some(inspected))
}

pub(super) fn verify_entries(
    inspected: &InspectedArtifact,
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<()> {
    checked(guard)?;
    inspected.directory.verify()?;
    checked(guard)?;
    if inspected.directory.entries(5)? != state::committed_entries() {
        return corrupt("sealed cache directory changed while it was being read");
    }
    inspected
        .marker
        .verify_while(inspected.directory.file(), &mut *guard)?;
    inspected
        .descriptor_file
        .verify_while(inspected.directory.file(), &mut *guard)?;
    inspected
        .payload
        .verify_while(inspected.directory.file(), &mut *guard)?;
    inspected
        .record_file
        .verify_while(inspected.directory.file(), &mut *guard)?;
    checked(guard)
}

fn open(
    directory: &BoundDirectory,
    name: &str,
    guard: &mut impl FnMut() -> bool,
) -> ArtifactResult<BoundFile> {
    checked(guard)?;
    let file = BoundFile::open(directory.file(), name.as_ref())?;
    checked(guard)?;
    file.verify_while(directory.file(), &mut *guard)?;
    Ok(file)
}

fn checked(guard: &mut impl FnMut() -> bool) -> ArtifactResult<()> {
    crate::cache::guard::check(guard)
}

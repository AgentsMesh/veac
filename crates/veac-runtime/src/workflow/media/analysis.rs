use std::path::Path;
use std::time::Instant;

use veac_artifact::{
    artifact_key, canonical_artifact_json_bounded, validate_artifact_json, ArtifactDescriptor,
    ArtifactRecord, ArtifactStore, MediaArtifactLimits, MediaArtifactRequest, MediaArtifactSpec,
    MAX_ANALYSIS_PAYLOAD_BYTES,
};

use super::{
    artifact_error, contract, source, GeneratedArtifact, WorkflowError, WorkflowErrorKind,
    WorkflowResult,
};

mod cache;

pub(super) fn store(
    store: &ArtifactStore,
    input: &Path,
    request: &MediaArtifactRequest,
    value: &serde_json::Value,
    limits: MediaArtifactLimits,
    deadline: Instant,
) -> WorkflowResult<GeneratedArtifact> {
    store_while(store, input, request, value, limits, || {
        Instant::now() < deadline
    })
}

fn store_while(
    store: &ArtifactStore,
    input: &Path,
    request: &MediaArtifactRequest,
    value: &serde_json::Value,
    limits: MediaArtifactLimits,
    mut guard: impl FnMut() -> bool,
) -> WorkflowResult<GeneratedArtifact> {
    active(&mut guard)?;
    contract(request.validate_with_limits(limits))?;
    active(&mut guard)?;
    source::verify_while(
        input,
        &request.source_identity,
        limits.max_source_bytes,
        &mut guard,
    )?;
    if !matches!(request.spec, MediaArtifactSpec::Analysis(_)) || !value.is_object() {
        return Err(WorkflowError::new(
            WorkflowErrorKind::UnsupportedOperation,
            "analysis storage requires an analysis spec and object payload",
        ));
    }
    active(&mut guard)?;
    contract(validate_artifact_json(value))?;
    active(&mut guard)?;
    let descriptor = contract(request.descriptor())?;
    let key = contract(artifact_key(&descriptor))?;
    let limit = limits.max_payload_bytes.min(MAX_ANALYSIS_PAYLOAD_BYTES);
    if let Some(cached) = store
        .open_verified_bounded_while(&key, &descriptor, limit, &mut guard)
        .map_err(artifact_error)?
    {
        cache::validate(&cached, limit, &mut guard)?;
        source::verify_while(
            input,
            &request.source_identity,
            limits.max_source_bytes,
            &mut guard,
        )?;
        return Ok(GeneratedArtifact {
            record: cached.record().clone(),
            cache_hit: true,
        });
    }
    active(&mut guard)?;
    let payload = contract(canonical_artifact_json_bounded(value, limit))?;
    active(&mut guard)?;
    source::verify_while(
        input,
        &request.source_identity,
        limits.max_source_bytes,
        &mut guard,
    )?;
    let record = store_fresh(store, &descriptor, &payload, &mut guard)?;
    source::verify_while(
        input,
        &request.source_identity,
        limits.max_source_bytes,
        &mut guard,
    )?;
    Ok(GeneratedArtifact {
        record,
        cache_hit: false,
    })
}

fn store_fresh(
    store: &ArtifactStore,
    descriptor: &ArtifactDescriptor,
    payload: &[u8],
    guard: impl FnMut() -> bool,
) -> WorkflowResult<ArtifactRecord> {
    store
        .put_while(descriptor, payload, guard)
        .map_err(artifact_error)
}

fn active(guard: &mut impl FnMut() -> bool) -> WorkflowResult<()> {
    if guard() {
        Ok(())
    } else {
        Err(WorkflowError::new(
            WorkflowErrorKind::ResourceLimit,
            "analysis storage exceeded its wall-clock limit",
        ))
    }
}

#[cfg(test)]
#[path = "analysis/tests.rs"]
mod tests;

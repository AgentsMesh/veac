use std::path::Path;
use std::time::Instant;

use veac_artifact::{
    artifact_key, AnalysisIngestionRequest, ArtifactDescriptor, ArtifactRecord, ArtifactStore,
    MediaArtifactLimits, MAX_ANALYSIS_PAYLOAD_BYTES,
};

use super::{
    artifact_error, contract, source, GeneratedArtifact, WorkflowError, WorkflowErrorKind,
    WorkflowResult,
};

mod cache;

pub(super) fn store(
    store: &ArtifactStore,
    input: &Path,
    request: &AnalysisIngestionRequest,
    limits: MediaArtifactLimits,
    deadline: Instant,
) -> WorkflowResult<GeneratedArtifact> {
    store_while(store, input, request, limits, || Instant::now() < deadline)
}

fn store_while(
    store: &ArtifactStore,
    input: &Path,
    request: &AnalysisIngestionRequest,
    limits: MediaArtifactLimits,
    mut guard: impl FnMut() -> bool,
) -> WorkflowResult<GeneratedArtifact> {
    active(&mut guard)?;
    contract(request.validate())?;
    active(&mut guard)?;
    source::verify_while(
        input,
        &request.source_identity,
        limits.max_source_bytes,
        &mut guard,
    )?;
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
    let payload = contract(request.result.canonical_bytes(limit))?;
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

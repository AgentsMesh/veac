use serde::Serialize;
use sha2::{Digest, Sha256};

use crate::{PlanCacheIdentity, PlanSource, ResolverFingerprint};

pub(crate) fn cache(
    source: &PlanSource,
    resolver: &ResolverFingerprint,
    temporal: &veac_ir::TemporalProgramLibrary,
) -> Result<PlanCacheIdentity, serde_json::Error> {
    Ok(PlanCacheIdentity {
        project_semantic_sha256: source.semantic_hash.clone(),
        project_snapshot_sha256: source.snapshot_hash.clone(),
        executable_manifest_sha256: digest(&source.executable)?,
        temporal_library_sha256: digest(temporal)?,
        resolver_sha256: digest(resolver)?,
    })
}

fn digest(value: &impl Serialize) -> Result<String, serde_json::Error> {
    let bytes = serde_json_canonicalizer::to_vec(value)?;
    Ok(crate::hash::hex_digest(Sha256::digest(bytes)))
}

#[cfg(test)]
#[path = "identity/tests.rs"]
mod tests;

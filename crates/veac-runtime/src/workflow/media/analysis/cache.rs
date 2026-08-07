use veac_artifact::{
    read_verified_source_bounded_while, AnalysisResultEnvelope, ArtifactParameters, ContentDigest,
    VerifiedArtifact,
};
use veac_ir::{HashAlgorithm, MediaIdentity};

use super::super::{artifact_error, contract, WorkflowError, WorkflowErrorKind, WorkflowResult};

pub(super) fn validate(
    artifact: &VerifiedArtifact,
    limit: u64,
    guard: &mut impl FnMut() -> bool,
) -> WorkflowResult<()> {
    if artifact.record().size_bytes > limit {
        return Err(WorkflowError::new(
            WorkflowErrorKind::ResourceLimit,
            "cached analysis artifact exceeds the active payload budget",
        ));
    }
    let identity = identity(&artifact.record().content);
    let bytes = read_verified_source_bounded_while(
        artifact.payload_path(),
        Some(&identity),
        limit,
        &mut *guard,
    )
    .map_err(artifact_error)?;
    super::active(guard)?;
    let value: AnalysisResultEnvelope = serde_json::from_slice(&bytes).map_err(|error| {
        WorkflowError::with_source(
            WorkflowErrorKind::Artifact,
            "cached analysis artifact is not a typed result envelope",
            error,
        )
    })?;
    super::active(guard)?;
    if value.validate().is_err() {
        return Err(WorkflowError::new(
            WorkflowErrorKind::Artifact,
            "cached analysis artifact violates its typed result contract",
        ));
    }
    super::active(guard)?;
    let canonical = contract(value.canonical_bytes(limit))?;
    super::active(guard)?;
    let ArtifactParameters::Analysis(parameters) = &artifact.descriptor().parameters else {
        return Err(WorkflowError::new(
            WorkflowErrorKind::Artifact,
            "cached analysis artifact has the wrong descriptor type",
        ));
    };
    if canonical != bytes || ContentDigest::sha256(&bytes) != parameters.result_digest {
        return Err(WorkflowError::new(
            WorkflowErrorKind::Artifact,
            "cached analysis artifact is not canonical JSON",
        ));
    }
    Ok(())
}

fn identity(digest: &ContentDigest) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: digest.value.clone(),
    }
}

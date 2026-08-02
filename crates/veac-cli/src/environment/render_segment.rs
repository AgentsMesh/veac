use std::path::Path;
use std::time::Instant;

use veac_artifact::{
    verify_source_prefix_bounded_while, ArtifactRecord, FullRenderSegmentContract,
    VerifiedSourcePrefix, MAX_ARTIFACT_PAYLOAD_BYTES, MAX_VERIFIED_SOURCE_PREFIX_BYTES,
};
use veac_ir::{HashAlgorithm, MediaIdentity, MediaProbeSnapshot, StreamChoice, StreamIntent};

use crate::error::{CliError, CliResult};

pub(crate) fn validate(
    path: &Path,
    contract: &FullRenderSegmentContract,
    record: &ArtifactRecord,
    deadline: Instant,
    probe: impl FnOnce(StreamIntent) -> CliResult<MediaProbeSnapshot>,
) -> CliResult {
    check_deadline(deadline)?;
    let expected = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: record.content.value.clone(),
    };
    verify(path, record, &expected, deadline)?;
    check_deadline(deadline)?;
    let snapshot = probe(intent(contract.has_audio()))?;
    let verified = verify(path, record, &expected, deadline)?;
    check_deadline(deadline)?;
    veac_runtime::workflow::validate_full_render_segment_snapshot(contract, record, &snapshot)
        .and_then(|_| {
            veac_runtime::workflow::validate_full_render_segment_prefix(contract, &verified.prefix)
        })
        .map_err(workflow_error)
}

fn verify(
    path: &Path,
    record: &ArtifactRecord,
    expected: &MediaIdentity,
    deadline: Instant,
) -> CliResult<VerifiedSourcePrefix> {
    let limit = record.size_bytes.max(1);
    if limit > MAX_ARTIFACT_PAYLOAD_BYTES {
        return Err(resource_error(
            "render-segment record exceeds the payload policy",
        ));
    }
    let source = verify_source_prefix_bounded_while(
        path,
        Some(expected),
        limit,
        MAX_VERIFIED_SOURCE_PREFIX_BYTES,
        || Instant::now() < deadline,
    )
    .map_err(|error| {
        if error.kind == veac_artifact::ArtifactErrorKind::ResourceLimit {
            resource_error(error.to_string())
        } else {
            postflight_error(error.to_string())
        }
    })?;
    if source.verified.size_bytes != record.size_bytes {
        return Err(postflight_error(
            "render-segment payload size differs from its record",
        ));
    }
    Ok(source)
}

fn intent(has_audio: bool) -> StreamIntent {
    StreamIntent {
        video: StreamChoice::Auto,
        audio: if has_audio {
            StreamChoice::Auto
        } else {
            StreamChoice::Disabled
        },
    }
}

fn check_deadline(deadline: Instant) -> CliResult {
    if Instant::now() >= deadline {
        return Err(resource_error(
            "render-segment postflight exceeded its wall budget",
        ));
    }
    Ok(())
}

fn postflight_error(message: impl Into<String>) -> CliError {
    CliError::new("RENDER_SEGMENT_POSTFLIGHT_FAILED", message)
}

fn resource_error(message: impl Into<String>) -> CliError {
    CliError::resource_limit("RENDER_SEGMENT_POSTFLIGHT_FAILED", message)
}

fn workflow_error(error: veac_runtime::workflow::WorkflowError) -> CliError {
    if error.kind == veac_runtime::workflow::WorkflowErrorKind::ResourceLimit {
        resource_error(error.to_string())
    } else {
        postflight_error(error.to_string())
    }
}

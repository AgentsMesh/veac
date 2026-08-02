use veac_provider::{
    canonical_provider_manifest_bytes, canonical_response_bytes, ProviderManifest,
    ProviderRequestEnvelope, ProviderResponseEnvelope, Validate,
};

use super::{WorkflowError, WorkflowErrorKind, WorkflowResult};

pub(super) fn decode_manifest(bytes: &[u8]) -> WorkflowResult<ProviderManifest> {
    let text = canonical_text(bytes, "provider manifest")?;
    veac_ir::reject_duplicate_json_keys(text).map_err(protocol_json)?;
    let manifest: ProviderManifest = serde_json::from_slice(bytes).map_err(protocol_json)?;
    manifest.validate().map_err(protocol_contract)?;
    if canonical_provider_manifest_bytes(&manifest).map_err(protocol_contract)? != bytes {
        return protocol("provider manifest stdout is not strict canonical JSON");
    }
    Ok(manifest)
}

pub(super) fn decode_response(
    bytes: &[u8],
    request: &ProviderRequestEnvelope,
) -> WorkflowResult<ProviderResponseEnvelope> {
    let text = canonical_text(bytes, "provider response")?;
    veac_ir::reject_duplicate_json_keys(text).map_err(protocol_json)?;
    let response: ProviderResponseEnvelope =
        serde_json::from_slice(bytes).map_err(protocol_json)?;
    response.validate_for(request).map_err(protocol_contract)?;
    if canonical_response_bytes(&response).map_err(protocol_contract)? != bytes {
        return protocol("provider response stdout is not strict canonical JSON");
    }
    Ok(response)
}

pub(super) fn input_contract(error: veac_provider::ProviderError) -> WorkflowError {
    WorkflowError::with_source(
        WorkflowErrorKind::InvalidContract,
        "provider request envelope is invalid",
        error,
    )
}

pub(super) fn protocol_contract(error: veac_provider::ProviderError) -> WorkflowError {
    WorkflowError::with_source(
        WorkflowErrorKind::ProtocolViolation,
        "provider protocol contract is invalid",
        error,
    )
}

fn canonical_text<'a>(bytes: &'a [u8], name: &str) -> WorkflowResult<&'a str> {
    std::str::from_utf8(bytes).map_err(|error| {
        WorkflowError::with_source(
            WorkflowErrorKind::ProtocolViolation,
            format!("{name} stdout is not UTF-8"),
            error,
        )
    })
}

fn protocol_json(error: serde_json::Error) -> WorkflowError {
    WorkflowError::with_source(
        WorkflowErrorKind::ProtocolViolation,
        "provider stdout is not strict JSON",
        error,
    )
}

fn protocol<T>(message: &str) -> WorkflowResult<T> {
    Err(WorkflowError::new(
        WorkflowErrorKind::ProtocolViolation,
        message,
    ))
}

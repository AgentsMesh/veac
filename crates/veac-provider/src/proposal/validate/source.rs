use crate::{
    request_hash, response_hash, ProviderEditProposal, ProviderError, ProviderErrorKind,
    ProviderResponseEnvelope, ProviderResult,
};

pub(super) fn validate(value: &ProviderEditProposal) -> ProviderResult<()> {
    if request_hash(&value.source_request)? != value.request_hash {
        return invalid("provider proposal request digest is inconsistent");
    }
    let response =
        ProviderResponseEnvelope::new(&value.source_request, value.source_output.clone())?;
    if response_hash(&response)? != value.response_hash {
        return invalid("provider proposal response digest is inconsistent");
    }
    let header = value.application_context.header();
    if value.project_timebase == 0
        || value.source_request.capability != value.application_context.capability()
        || header.project_revision != value.project_revision
        || header.operation_id != value.batch.operation_id
    {
        return invalid("provider proposal application context is inconsistent");
    }
    Ok(())
}

fn invalid<T>(message: &str) -> ProviderResult<T> {
    Err(ProviderError::new(
        ProviderErrorKind::InvalidContract,
        message,
    ))
}

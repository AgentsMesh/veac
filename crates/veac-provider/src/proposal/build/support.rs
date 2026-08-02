use veac_ir::{Clip, EditOperation, Precondition, ProjectEnvelope, Track};

use super::super::{ApplicationContext, ProposalEvidence};
use crate::{ProviderError, ProviderErrorKind, ProviderRequestEnvelope, ProviderResult};

pub(super) struct BuiltApplication {
    pub operations: Vec<EditOperation>,
    pub evidence: Vec<ProposalEvidence>,
    pub preconditions: Vec<Precondition>,
}

impl BuiltApplication {
    pub fn new(operations: Vec<EditOperation>, evidence: Vec<ProposalEvidence>) -> Self {
        Self {
            operations,
            evidence,
            preconditions: Vec::new(),
        }
    }
}

pub(super) fn validate_context(
    project: &ProjectEnvelope,
    request: &ProviderRequestEnvelope,
    context: &ApplicationContext,
) -> ProviderResult<()> {
    let header = context.header();
    if request.capability != context.capability() {
        return invalid("application context does not match the provider capability");
    }
    if header.project_revision != project.project.revision
        || header.project_revision > veac_ir::MAX_SAFE_INTEGER
    {
        return invalid("provider proposal revision does not match the project");
    }
    if veac_ir::OperationId::new(header.operation_id.as_str()).is_err() {
        return invalid("provider proposal operation ID is invalid");
    }
    Ok(())
}

pub(super) fn clip<'a>(
    project: &'a ProjectEnvelope,
    id: &veac_ir::ItemId,
) -> ProviderResult<(&'a Track, &'a Clip)> {
    project
        .project
        .sequences
        .iter()
        .flat_map(|sequence| &sequence.tracks)
        .find_map(|track| {
            track
                .clips
                .iter()
                .find(|clip| clip.id == *id)
                .map(|clip| (track, clip))
        })
        .ok_or_else(|| invalid_error("provider proposal target clip does not exist"))
}

pub(super) fn index(value: usize) -> ProviderResult<u32> {
    value
        .try_into()
        .map_err(|_| invalid_error("provider proposal has too many operations"))
}

pub(super) fn invalid<T>(message: &str) -> ProviderResult<T> {
    Err(invalid_error(message))
}

pub(super) fn unsupported<T>(message: &str) -> ProviderResult<T> {
    Err(ProviderError::new(
        ProviderErrorKind::UnsupportedApplication,
        message,
    ))
}

fn invalid_error(message: &str) -> ProviderError {
    ProviderError::new(ProviderErrorKind::InvalidContract, message)
}

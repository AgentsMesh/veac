use veac_ir::{EditBatch, EditOutcome, ProjectEnvelope};

use super::{ApplicationContext, ProviderEditProposal};
use crate::{
    request_hash, response_hash, ProviderError, ProviderErrorKind, ProviderOutput,
    ProviderRequestEnvelope, ProviderResponseEnvelope, ProviderResult, Validate,
};

mod annotation;
mod asr;
mod color;
mod crop;
mod matte;
mod media;
mod media_insert;
mod media_replace;
mod multicam;
mod retouch;
mod separation;
mod support;
mod transform;
mod translation;
mod visual_media;

pub fn propose_edit(
    project: &ProjectEnvelope,
    request: &ProviderRequestEnvelope,
    response: &ProviderResponseEnvelope,
    context: &ApplicationContext,
) -> ProviderResult<ProviderEditProposal> {
    veac_ir::validate(project).map_err(|error| {
        ProviderError::with_source(
            ProviderErrorKind::InvalidContract,
            "provider proposal project is invalid",
            error,
        )
    })?;
    response.validate_for(request)?;
    support::validate_context(project, request, context)?;
    let request_hash = request_hash(request)?;
    let response_hash = response_hash(response)?;
    let provenance = veac_ir::AnnotationProvenance {
        producer: request.provider.annotation_producer(),
        request_sha256: request_hash.value.clone(),
        response_sha256: response_hash.value.clone(),
    };
    let built = dispatch(
        project,
        &request.request,
        &response.output,
        context,
        &provenance,
    )?;
    let header = context.header();
    let proposal = ProviderEditProposal {
        request_hash,
        response_hash,
        source_request: Box::new(request.clone()),
        application_context: Box::new(context.clone()),
        project_revision: header.project_revision,
        project_timebase: project.project.timebase,
        source_output: response.output.clone(),
        evidence: built.evidence,
        batch: EditBatch {
            operation_id: header.operation_id.clone(),
            base_revision: header.project_revision,
            atomic: true,
            preconditions: built.preconditions,
            operations: built.operations,
        },
    };
    proposal.validate()?;
    match veac_ir::apply_edit_batch(project, &proposal.batch) {
        EditOutcome::Applied { .. } => Ok(proposal),
        outcome => Err(ProviderError::new(
            ProviderErrorKind::InvalidContract,
            format!("provider proposal is not applicable to the project snapshot: {outcome:?}"),
        )),
    }
}

fn dispatch(
    project: &ProjectEnvelope,
    request: &crate::ProviderRequest,
    output: &ProviderOutput,
    context: &ApplicationContext,
    provenance: &veac_ir::AnnotationProvenance,
) -> ProviderResult<support::BuiltApplication> {
    match (request, output, context) {
        (_, ProviderOutput::Asr(result), ApplicationContext::AsrCaptions(value)) => {
            asr::build(project, result, value)
        }
        (_, ProviderOutput::Translation(result), ApplicationContext::Translation(value)) => {
            translation::build(project, result, value)
        }
        (
            _,
            output @ (ProviderOutput::LanguageDetection(_)
            | ProviderOutput::SceneDetection(_)
            | ProviderOutput::BeatDetection(_)
            | ProviderOutput::SilenceDetection(_)
            | ProviderOutput::FillerDetection(_)
            | ProviderOutput::HighlightDetection(_)),
            ApplicationContext::AnalysisAnnotations(value),
        ) => annotation::build(project, output, value, provenance),
        (
            crate::ProviderRequest::TextToSpeech(request),
            ProviderOutput::TextToSpeech(result),
            ApplicationContext::TextToSpeech(value),
        ) => media_insert::tts(project, request, result, value),
        (
            crate::ProviderRequest::Dubbing(request),
            ProviderOutput::Dubbing(result),
            ApplicationContext::Dubbing(value),
        ) => media_insert::dubbing(project, request, result, value),
        (_, ProviderOutput::MotionTracking(result), ApplicationContext::MotionTracking(value)) => {
            transform::tracking(project, result, value)
        }
        (
            crate::ProviderRequest::Stabilization(request),
            ProviderOutput::Stabilization(result),
            ApplicationContext::Stabilization(value),
        ) => transform::stabilization(project, request, result, value),
        (
            crate::ProviderRequest::Segmentation(request),
            ProviderOutput::Segmentation(result),
            ApplicationContext::Segmentation(value),
        ) => matte::segmentation(project, request, result, value),
        (
            crate::ProviderRequest::Matte(_),
            ProviderOutput::Matte(result),
            ApplicationContext::Matte(value),
        ) => matte::matte(project, result, value),
        (
            crate::ProviderRequest::Denoise(request),
            ProviderOutput::Denoise(result),
            ApplicationContext::Denoise(value),
        ) => media_replace::denoise(project, request, result, value),
        (
            crate::ProviderRequest::VocalSeparation(request),
            ProviderOutput::VocalSeparation(result),
            ApplicationContext::VocalSeparation(value),
        ) => separation::build(project, request, result, value),
        (
            crate::ProviderRequest::AutoReframe(request),
            ProviderOutput::AutoReframe(result),
            ApplicationContext::AutoReframe(value),
        ) => crop::reframe(project, request, result, value),
        (
            crate::ProviderRequest::Retouch(request),
            ProviderOutput::Retouch(result),
            ApplicationContext::Retouch(value),
        ) => retouch::build(project, request, result, value),
        (
            crate::ProviderRequest::Removal(request),
            ProviderOutput::Removal(result),
            ApplicationContext::Removal(value),
        ) => media_replace::removal(project, request, result, value),
        (_, ProviderOutput::ColorMatch(result), ApplicationContext::ColorMatch(value)) => {
            color::build(project, result, value)
        }
        (
            crate::ProviderRequest::MulticamSync(request),
            ProviderOutput::MulticamSync(result),
            ApplicationContext::MulticamSync(value),
        ) => multicam::build(project, request, result, value),
        _ => support::invalid("application context does not match the provider output"),
    }
}

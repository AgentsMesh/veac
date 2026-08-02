use veac_ir::EditOperation;

use super::{ProposalEvidence, ProviderEditProposal};
use crate::ProviderOutput;

#[path = "evidence_validate/annotation.rs"]
mod annotation;
#[path = "evidence_validate/artifact.rs"]
mod artifact;
#[path = "evidence_validate/color.rs"]
mod color;
#[path = "evidence_validate/context.rs"]
mod context;
#[path = "evidence_validate/multicam.rs"]
mod multicam;
#[path = "evidence_validate/visual.rs"]
mod visual;

pub(super) fn matches(
    evidence: &ProposalEvidence,
    proposal: &ProviderEditProposal,
    operation: &EditOperation,
) -> bool {
    if !context::matches(evidence, &proposal.application_context, operation) {
        return false;
    }
    let source = &proposal.source_output;
    match evidence {
        ProposalEvidence::AsrSegment { segment_id, .. } => {
            let ProviderOutput::Asr(result) = source else {
                return false;
            };
            let Some(segment) = result.segments.iter().find(|value| value.id == *segment_id) else {
                return false;
            };
            matches!(
                operation,
                EditOperation::InsertClip { clip, .. }
                    if matches!(&clip.source, veac_ir::ClipSource::Caption { text, .. } if text == &segment.text)
            )
        }
        ProposalEvidence::TranslationUnit { unit_id, .. } => {
            let ProviderOutput::Translation(result) = source else {
                return false;
            };
            let Some(unit) = result.units.iter().find(|value| value.id == *unit_id) else {
                return false;
            };
            matches!(operation, EditOperation::SetText { text, .. } if text == &unit.text)
        }
        ProposalEvidence::AnalysisAnnotation {
            kind,
            result_index,
            annotation_id,
            ..
        } => annotation::matches(*kind, *result_index, annotation_id, proposal, operation),
        ProposalEvidence::ArtifactMaterial {
            artifact_key,
            artifact_role,
            material_id,
            ..
        } => artifact::material(source, operation, artifact_key, artifact_role, material_id),
        ProposalEvidence::GeneratedAudioClip {
            artifact_key,
            artifact_role,
            material_id,
            ..
        } => artifact::generated_audio(source, operation, artifact_key, artifact_role, material_id),
        ProposalEvidence::ReplacedMedia {
            artifact_key,
            artifact_role,
            material_id,
            ..
        } => artifact::replacement(source, operation, artifact_key, artifact_role, material_id),
        ProposalEvidence::SeparatedStemClip {
            artifact_key,
            artifact_role,
            material_id,
            kind,
            label,
            ..
        } => artifact::stem(
            source,
            operation,
            artifact_key,
            artifact_role,
            material_id,
            *kind,
            label,
        ),
        ProposalEvidence::ArtifactVisualClip {
            artifact_key,
            artifact_role,
            material_id,
            ..
        } => artifact::visual_clip(source, operation, artifact_key, artifact_role, material_id),
        ProposalEvidence::AppliedTrackMatte {
            artifact_key,
            artifact_role,
            matte_clip_id,
            ..
        } => artifact::track_matte(
            source,
            operation,
            artifact_key,
            artifact_role,
            matte_clip_id,
        ),
        evidence @ ProposalEvidence::RetouchApply { .. } => {
            artifact::retouch(source, operation, evidence, proposal.project_timebase)
        }
        ProposalEvidence::TransformSamples {
            sample_indices,
            component,
            ..
        } => visual::transform(source, operation, sample_indices, *component),
        ProposalEvidence::CropSamples {
            sample_indices,
            time,
            timebase,
            keyframe_id_prefix,
            ..
        } => visual::crop(
            source,
            operation,
            sample_indices,
            *time,
            *timebase,
            keyframe_id_prefix,
        ),
        ProposalEvidence::ColorPipeline { .. } => color::pipeline(source, operation),
        ProposalEvidence::ColorAdjustEffect { .. } => color::effect(source, operation),
        ProposalEvidence::MulticamSync {
            group_id,
            angle_indices,
            ..
        } => multicam::matches(source, operation, group_id, angle_indices),
    }
}

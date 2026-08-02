use veac_ir::{EditOperation, ItemId, RelationEndpoint, RelationKind, StructureEdit};

use crate::{ApplicationContext, ProposalEvidence};

#[path = "context/media.rs"]
mod media;
#[path = "context/retouch.rs"]
mod retouch;
pub(super) fn matches(
    evidence: &ProposalEvidence,
    application: &ApplicationContext,
    operation: &EditOperation,
) -> bool {
    match evidence {
        ProposalEvidence::AsrSegment { .. } => asr(application, operation),
        ProposalEvidence::TranslationUnit { unit_id, .. } => {
            translation(application, operation, unit_id)
        }
        ProposalEvidence::AnalysisAnnotation { .. } => {
            matches!(application, ApplicationContext::AnalysisAnnotations(_))
        }
        ProposalEvidence::ArtifactMaterial { material_id, .. } => {
            media::material(application, operation, material_id)
        }
        ProposalEvidence::GeneratedAudioClip { material_id, .. } => {
            media::generated_audio(application, operation, material_id)
        }
        ProposalEvidence::ReplacedMedia { material_id, .. } => {
            media::replacement(application, operation, material_id)
        }
        ProposalEvidence::SeparatedStemClip { material_id, .. } => {
            media::stem(application, operation, material_id)
        }
        ProposalEvidence::ArtifactVisualClip { material_id, .. } => {
            media::visual_clip(application, operation, material_id)
        }
        ProposalEvidence::AppliedTrackMatte { .. } => matte_target(application, operation),
        evidence @ ProposalEvidence::RetouchApply { .. } => {
            retouch::matches(application, operation, evidence)
        }
        ProposalEvidence::TransformSamples { .. } => transform(application, operation),
        ProposalEvidence::CropSamples {
            time,
            keyframe_id_prefix,
            ..
        } => crop_samples(application, operation, *time, keyframe_id_prefix),
        ProposalEvidence::ColorPipeline { .. } => color_pipeline(application, operation),
        ProposalEvidence::ColorAdjustEffect { .. } => color_effect(application, operation),
        ProposalEvidence::MulticamSync { group_id, .. } => {
            multicam(application, operation, group_id)
        }
    }
}

fn asr(application: &ApplicationContext, operation: &EditOperation) -> bool {
    matches!(
        (application, operation),
        (
            ApplicationContext::AsrCaptions(context),
            EditOperation::InsertClip { sequence_id, track_id, .. }
        ) if *sequence_id == context.sequence_id && *track_id == context.track_id
    )
}

fn translation(application: &ApplicationContext, operation: &EditOperation, unit: &str) -> bool {
    let ApplicationContext::Translation(context) = application else {
        return false;
    };
    let EditOperation::SetText { clip_id, .. } = operation else {
        return false;
    };
    context
        .bindings
        .iter()
        .any(|binding| binding.unit_id == unit && binding.clip_id == *clip_id)
}

fn matte_target(application: &ApplicationContext, operation: &EditOperation) -> bool {
    let (sequence_id, producer_id, expected_consumer, mode, invert) = match application {
        ApplicationContext::Segmentation(value) | ApplicationContext::Matte(value) => (
            &value.matte.sequence_id,
            &value.matte.clip_id,
            RelationEndpoint::item(value.target_clip_id.clone()),
            value.mode,
            value.invert,
        ),
        ApplicationContext::Retouch(value) => {
            let Some(matte) = &value.matte else {
                return false;
            };
            (
                &value.sequence_id,
                &matte.insertion.clip_id,
                RelationEndpoint::apply(value.apply_id.clone()),
                matte.mode,
                matte.invert,
            )
        }
        _ => return false,
    };
    matches!(
        operation,
        EditOperation::EditStructure {
            edit: StructureEdit::InsertRelation { relation }
        } if relation.sequence_id == *sequence_id
            && matches!(&relation.kind, RelationKind::Matte {
                producer,
                consumer,
                parameters,
            } if producer.item_id() == Some(producer_id)
                && consumer == &expected_consumer
                && parameters.mode == mode
                && parameters.invert == invert)
    )
}

fn transform(application: &ApplicationContext, operation: &EditOperation) -> bool {
    let target = match application {
        ApplicationContext::MotionTracking(value) | ApplicationContext::Stabilization(value) => {
            &value.clip_id
        }
        _ => return false,
    };
    visual_property_target(operation, target)
}

fn crop_samples(
    application: &ApplicationContext,
    operation: &EditOperation,
    time: crate::ClipTimeBinding,
    prefix: &str,
) -> bool {
    let (context_time, context_prefix, target) = match application {
        ApplicationContext::Stabilization(value) => (
            value.time,
            value.keyframe_id_prefix.as_str(),
            &value.clip_id,
        ),
        ApplicationContext::AutoReframe(value) => (
            value.time,
            value.keyframe_id_prefix.as_str(),
            &value.clip_id,
        ),
        _ => return false,
    };
    context_time == time && context_prefix == prefix && visual_property_target(operation, target)
}

fn color_pipeline(application: &ApplicationContext, operation: &EditOperation) -> bool {
    let ApplicationContext::ColorMatch(context) = application else {
        return false;
    };
    visual_property_target(operation, &context.clip_id)
}

fn color_effect(application: &ApplicationContext, operation: &EditOperation) -> bool {
    let ApplicationContext::ColorMatch(context) = application else {
        return false;
    };
    matches!(
        operation,
        EditOperation::AddEffect { clip_id, effect, .. }
            if *clip_id == context.clip_id && effect.id == context.effect_id
    )
}

fn multicam(
    application: &ApplicationContext,
    operation: &EditOperation,
    evidence_group: &veac_ir::MulticamGroupId,
) -> bool {
    let ApplicationContext::MulticamSync(context) = application else {
        return false;
    };
    matches!(
        operation,
        EditOperation::SetMulticamGroup { group_id, .. }
            if group_id == evidence_group && *group_id == context.group_id
    )
}

fn visual_property_target(operation: &EditOperation, target: &ItemId) -> bool {
    matches!(operation, EditOperation::SetVisualProperty { clip_id, .. } if clip_id == target)
}

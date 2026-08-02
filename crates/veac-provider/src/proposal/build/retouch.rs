use veac_ir::{Apply, ApplyTarget, EditOperation, Precondition, ProjectEnvelope, StructureEdit};

use crate::{
    OperationBinding, ProposalEvidence, ProviderResult, RetouchApplication, RetouchRequest,
    RetouchResult,
};

use super::media::{self, RequiredStream};
use super::support::{self, BuiltApplication};
use super::visual_media;

mod controls;
mod matte;

pub(super) fn build(
    project: &ProjectEnvelope,
    request: &RetouchRequest,
    result: &RetouchResult,
    context: &RetouchApplication,
) -> ProviderResult<BuiltApplication> {
    let (source_track, source_clip) = media::source_clip(
        project,
        &context.target_clip_id,
        &request.video,
        RequiredStream::Video,
        true,
    )?;
    let (sequence, target_track, target_clip) =
        visual_media::target(project, &context.target_clip_id)?;
    let record_range = media::normalized_range(project, context.record_range)?;
    if sequence.id != context.sequence_id
        || record_range != target_clip.record_range
        || result.controls != request.controls
    {
        return support::invalid("retouch target, range, or controls do not match the request");
    }
    validate_identity(sequence, context)?;
    let compiled = controls::compile(
        &context.effects,
        &context.controls,
        &result.controls,
        context.time,
        project.project.timebase,
    )?;
    let mut built = BuiltApplication::new(Vec::new(), Vec::new());
    let matte =
        matte::append_optional(project, result, context, sequence, record_range, &mut built)?;
    append_apply(
        context,
        record_range,
        compiled,
        project.project.timebase,
        &mut built,
    )?;
    if let Some(artifact) = matte {
        matte::append_relation(context, artifact, &mut built)?;
    }
    built
        .preconditions
        .extend(media::source_preconditions(source_track, source_clip));
    if target_track.id != source_track.id {
        built.preconditions.push(Precondition::TrackUnlocked {
            track_id: target_track.id.clone(),
        });
    }
    Ok(built)
}

fn validate_identity(
    sequence: &veac_ir::Sequence,
    context: &RetouchApplication,
) -> ProviderResult<()> {
    if context.before_apply_id.is_some() && context.after_apply_id.is_some()
        || sequence
            .applies
            .iter()
            .any(|apply| apply.id == context.apply_id)
    {
        return support::invalid("retouch apply identity or placement is invalid");
    }
    for anchor in [&context.before_apply_id, &context.after_apply_id]
        .into_iter()
        .flatten()
    {
        if !sequence.applies.iter().any(|apply| &apply.id == anchor) {
            return support::invalid("retouch apply anchor does not exist in its sequence");
        }
    }
    Ok(())
}

fn append_apply(
    context: &RetouchApplication,
    record_range: veac_ir::TimeRange,
    compiled: controls::CompiledControls,
    timebase: u32,
    built: &mut BuiltApplication,
) -> ProviderResult<()> {
    let stage_ids = compiled
        .stages
        .iter()
        .map(|stage| stage.id.clone())
        .collect();
    let operation = EditOperation::EditStructure {
        edit: StructureEdit::InsertApply {
            sequence_id: context.sequence_id.clone(),
            apply: Box::new(Apply {
                id: context.apply_id.clone(),
                enabled: true,
                record_range,
                target: ApplyTarget::ItemSet {
                    item_ids: vec![context.target_clip_id.clone()],
                },
                stages: compiled.stages,
                mix: context.apply_mix.clone(),
            }),
            before_id: context.before_apply_id.clone(),
            after_id: context.after_apply_id.clone(),
        },
    };
    let index = support::index(built.operations.len())?;
    built.evidence.push(ProposalEvidence::RetouchApply {
        operation: OperationBinding::new(index, &operation)?,
        target_clip_id: context.target_clip_id.clone(),
        apply_id: context.apply_id.clone(),
        apply_stage_ids: stage_ids,
        time: context.time,
        timebase,
        controls: compiled.evidence,
    });
    built.operations.push(operation);
    Ok(())
}

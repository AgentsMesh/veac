use veac_ir::{
    Apply, ApplyOperation, ApplyStage, ApplyStageId, ApplyTarget, EditOperation,
    EffectParameterValue, StructureEdit,
};

use crate::{
    ApplicationContext, RetouchApplication, RetouchControlEvidence, RetouchEffectApplication,
};

pub(super) fn matches(
    application: &ApplicationContext,
    operation: &EditOperation,
    evidence: &crate::ProposalEvidence,
) -> bool {
    let crate::ProposalEvidence::RetouchApply {
        target_clip_id,
        apply_id,
        apply_stage_ids,
        time,
        timebase,
        controls,
        ..
    } = evidence
    else {
        return false;
    };
    let ApplicationContext::Retouch(context) = application else {
        return false;
    };
    let EditOperation::EditStructure {
        edit:
            StructureEdit::InsertApply {
                sequence_id,
                apply,
                before_id,
                after_id,
            },
    } = operation
    else {
        return false;
    };
    target_clip_id == &context.target_clip_id
        && apply_id == &context.apply_id
        && *time == context.time
        && sequence_id == &context.sequence_id
        && before_id == &context.before_apply_id
        && after_id == &context.after_apply_id
        && apply_matches(context, apply, apply_stage_ids, *timebase, controls)
}

fn apply_matches(
    context: &RetouchApplication,
    apply: &Apply,
    stage_ids: &[ApplyStageId],
    timebase: u32,
    controls: &[RetouchControlEvidence],
) -> bool {
    let range = crate::proposal::time::range_to_timebase(context.record_range, timebase);
    apply.id == context.apply_id
        && apply.enabled
        && apply.record_range.start.timescale == timebase
        && apply.record_range.duration.timescale == timebase
        && range.is_ok_and(|range| range == apply.record_range)
        && apply.target
            == (ApplyTarget::ItemSet {
                item_ids: vec![context.target_clip_id.clone()],
            })
        && apply.mix == context.apply_mix
        && stage_ids
            == context
                .effects
                .iter()
                .map(|value| value.stage_id.clone())
                .collect::<Vec<_>>()
        && controls_match(context, controls)
        && apply.stages.len() == context.effects.len()
        && context
            .effects
            .iter()
            .zip(&apply.stages)
            .all(|(template, stage)| stage_matches(template, stage, controls))
}

fn controls_match(context: &RetouchApplication, evidence: &[RetouchControlEvidence]) -> bool {
    context.controls.len() == evidence.len()
        && context.controls.iter().zip(evidence).all(|(left, right)| {
            let stage = context
                .effects
                .iter()
                .find(|value| value.effect.id == left.effect_id);
            left.control == right.control
                && left.effect_id == right.effect_id
                && left.effect_parameter == right.effect_parameter
                && left.keyframe_id_prefix == right.keyframe_id_prefix
                && stage.is_some_and(|value| value.stage_id == right.apply_stage_id)
        })
}

fn stage_matches(
    template: &RetouchEffectApplication,
    stage: &ApplyStage,
    controls: &[RetouchControlEvidence],
) -> bool {
    let ApplyOperation::Effect { effect } = &stage.operation else {
        return false;
    };
    if !stage.enabled
        || stage.id != template.stage_id
        || stage.active_range != template.active_range
        || effect.id != template.effect.id
        || effect.effect.kind() != template.effect.effect.kind()
        || effect.enabled != template.effect.enabled
        || effect.enable_range != template.effect.enable_range
    {
        return false;
    }
    let mapped = controls
        .iter()
        .filter(|value| value.effect_id == effect.id)
        .map(|value| value.effect_parameter)
        .collect::<Vec<_>>();
    let mut base = effect.effect.clone();
    for parameter in mapped {
        let Some(value) = template.effect.effect.curve(parameter).cloned() else {
            return false;
        };
        if base
            .set_parameter(parameter, EffectParameterValue::Curve(value))
            .is_none()
        {
            return false;
        }
    }
    base == template.effect.effect
}

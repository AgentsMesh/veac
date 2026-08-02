use veac_ir::{
    Animatable, ApplyOperation, EditOperation, Interpolation, ParameterValue, StructureEdit,
};

use crate::{
    ClipTimeBinding, ProposalEvidence, ProviderOutput, RetouchControlEvidence, RetouchResult,
    ScalarSample,
};

pub(in crate::proposal::evidence_validate) fn matches(
    source: &ProviderOutput,
    operation: &EditOperation,
    evidence: &ProposalEvidence,
    project_timebase: u32,
) -> bool {
    let ProposalEvidence::RetouchApply {
        apply_stage_ids,
        time,
        timebase,
        controls,
        ..
    } = evidence
    else {
        return false;
    };
    let ProviderOutput::Retouch(result) = source else {
        return false;
    };
    let EditOperation::EditStructure {
        edit: StructureEdit::InsertApply { apply, .. },
    } = operation
    else {
        return false;
    };
    *timebase == project_timebase
        && complete_controls(result, controls)
        && apply_stage_ids
            == &apply
                .stages
                .iter()
                .map(|stage| stage.id.clone())
                .collect::<Vec<_>>()
        && controls.iter().all(|binding| {
            apply.stages.iter().any(|stage| {
                stage.id == binding.apply_stage_id
                    && matches!(&stage.operation, ApplyOperation::Effect { effect }
                        if effect.id == binding.effect_id
                            && parameter(effect, result, binding, *time, *timebase))
            })
        })
}

fn complete_controls(result: &RetouchResult, controls: &[RetouchControlEvidence]) -> bool {
    controls.len() == result.controls.len()
        && controls
            .windows(2)
            .all(|pair| pair[0].control < pair[1].control)
        && controls.iter().all(|binding| {
            result
                .controls
                .iter()
                .find(|curve| curve.parameter == binding.control)
                .is_some_and(|curve| complete_indices(&binding.sample_indices, curve.samples.len()))
        })
}

fn parameter(
    effect: &veac_ir::EffectInstance,
    result: &RetouchResult,
    binding: &RetouchControlEvidence,
    time: ClipTimeBinding,
    timebase: u32,
) -> bool {
    let Some(source) = result
        .controls
        .iter()
        .find(|curve| curve.parameter == binding.control)
    else {
        return false;
    };
    let Some(ParameterValue::NumberCurve {
        value: Animatable::Keyframes { keyframes },
    }) = effect.parameters.get(&binding.effect_parameter)
    else {
        return false;
    };
    keyframes.len() == source.samples.len()
        && keyframes
            .iter()
            .zip(&source.samples)
            .enumerate()
            .all(|(index, (keyframe, sample))| {
                sample_matches(keyframe, sample, time, timebase, binding, index)
            })
}

fn sample_matches(
    keyframe: &veac_ir::Keyframe<f64>,
    sample: &ScalarSample,
    time: ClipTimeBinding,
    timebase: u32,
    binding: &RetouchControlEvidence,
    index: usize,
) -> bool {
    keyframe.value == sample.value
        && matches!(keyframe.interpolation, Interpolation::Linear)
        && super::super::super::time::clip_time(sample.time, time, timebase)
            .is_ok_and(|mapped| mapped == keyframe.time)
        && veac_ir::KeyframeId::new(format!("{}_{:04}", binding.keyframe_id_prefix, index + 1))
            .is_ok_and(|id| id == keyframe.id)
}

fn complete_indices(values: &[u32], len: usize) -> bool {
    values.len() == len
        && values
            .iter()
            .enumerate()
            .all(|(index, value)| usize::try_from(*value).ok() == Some(index))
}

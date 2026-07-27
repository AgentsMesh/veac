use std::collections::BTreeSet;

use veac_ir::{
    Animatable, ApplyOperation, ApplyStage, EffectInstance, Interpolation, Keyframe, KeyframeId,
    ParameterValue,
};

use crate::{
    ClipTimeBinding, ParameterCurve, ProviderError, ProviderErrorKind, ProviderResult,
    RetouchControlApplication, RetouchControlEvidence, RetouchEffectApplication,
};

pub(super) struct CompiledControls {
    pub stages: Vec<ApplyStage>,
    pub evidence: Vec<RetouchControlEvidence>,
}

pub(super) fn compile(
    templates: &[RetouchEffectApplication],
    bindings: &[RetouchControlApplication],
    curves: &[ParameterCurve],
    time: ClipTimeBinding,
    timebase: u32,
) -> ProviderResult<CompiledControls> {
    validate_shape(templates, bindings, curves)?;
    let mut applications = templates.to_vec();
    let mut evidence = Vec::with_capacity(bindings.len());
    for binding in bindings {
        let curve = curves
            .iter()
            .find(|curve| curve.parameter == binding.control)
            .ok_or_else(|| invalid_error("retouch control binding is incomplete"))?;
        evidence.push(compile_binding(
            &mut applications,
            binding,
            curve,
            time,
            timebase,
        )?);
    }
    let stages = applications
        .into_iter()
        .map(|application| ApplyStage {
            id: application.stage_id,
            enabled: true,
            active_range: application.active_range,
            operation: ApplyOperation::Effect {
                effect: application.effect,
            },
        })
        .collect();
    Ok(CompiledControls { stages, evidence })
}

fn validate_shape(
    templates: &[RetouchEffectApplication],
    bindings: &[RetouchControlApplication],
    curves: &[ParameterCurve],
) -> ProviderResult<()> {
    let ordered = bindings
        .windows(2)
        .all(|pair| pair[0].control < pair[1].control);
    if templates.is_empty() || bindings.len() != curves.len() || !ordered {
        return invalid("retouch effects and control bindings must be complete and ordered");
    }
    let mut stages = BTreeSet::new();
    let mut effects = BTreeSet::new();
    for template in templates {
        if !stages.insert(template.stage_id.clone())
            || !effects.insert(template.effect.id.clone())
            || template.effect.enable_range.is_some()
        {
            return invalid("retouch apply stage and effect identities must be unique");
        }
    }
    let mut targets = BTreeSet::new();
    if bindings.iter().any(|binding| {
        !targets.insert((binding.effect_id.clone(), binding.effect_parameter.clone()))
    }) {
        return invalid("retouch controls must target unique effect parameters");
    }
    Ok(())
}

fn compile_binding(
    applications: &mut [RetouchEffectApplication],
    binding: &RetouchControlApplication,
    curve: &ParameterCurve,
    time: ClipTimeBinding,
    timebase: u32,
) -> ProviderResult<RetouchControlEvidence> {
    let application = applications
        .iter_mut()
        .find(|application| application.effect.id == binding.effect_id)
        .ok_or_else(|| invalid_error("retouch control references an unknown effect"))?;
    let parameter = parameter_spec(&application.effect, &binding.effect_parameter)?;
    if application
        .effect
        .parameters
        .contains_key(&binding.effect_parameter)
    {
        return invalid("retouch control target must not be pre-populated");
    }
    let (value, sample_indices) = curve_value(curve, binding, time, timebase)?;
    if !veac_ir::parameter_matches(parameter, &value) {
        return unsupported("retouch control values exceed the effect contract");
    }
    application
        .effect
        .parameters
        .insert(binding.effect_parameter.clone(), value);
    Ok(RetouchControlEvidence {
        control: binding.control.clone(),
        apply_stage_id: application.stage_id.clone(),
        effect_id: binding.effect_id.clone(),
        effect_parameter: binding.effect_parameter.clone(),
        keyframe_id_prefix: binding.keyframe_id_prefix.clone(),
        sample_indices,
    })
}

fn parameter_spec(
    effect: &EffectInstance,
    parameter: &str,
) -> ProviderResult<veac_ir::ParameterSpec> {
    let specification = veac_ir::built_in_effect(&effect.effect_type)
        .filter(|_| effect.effect_type.starts_with("video."))
        .ok_or_else(|| unsupported_error("retouch requires a canonical video effect"))?;
    specification
        .parameters
        .iter()
        .copied()
        .find(|value| value.name == parameter && value.supports_curve)
        .ok_or_else(|| unsupported_error("retouch parameter does not support numeric curves"))
}

fn curve_value(
    curve: &ParameterCurve,
    binding: &RetouchControlApplication,
    time: ClipTimeBinding,
    timebase: u32,
) -> ProviderResult<(ParameterValue, Vec<u32>)> {
    let mut keyframes = Vec::with_capacity(curve.samples.len());
    let mut indices = Vec::with_capacity(curve.samples.len());
    for (index, sample) in curve.samples.iter().enumerate() {
        let ordinal = index
            .checked_add(1)
            .and_then(|value| u32::try_from(value).ok())
            .ok_or_else(|| invalid_error("retouch control has too many samples"))?;
        let id = KeyframeId::new(format!("{}_{ordinal:04}", binding.keyframe_id_prefix))
            .map_err(|_| invalid_error("retouch keyframe prefix is invalid"))?;
        keyframes.push(Keyframe {
            id,
            time: super::super::super::time::clip_time(sample.time, time, timebase)?,
            value: sample.value,
            interpolation: Interpolation::Linear,
        });
        indices.push(ordinal - 1);
    }
    Ok((
        ParameterValue::NumberCurve {
            value: Animatable::Keyframes { keyframes },
        },
        indices,
    ))
}

fn invalid<T>(message: &str) -> ProviderResult<T> {
    Err(invalid_error(message))
}

fn unsupported<T>(message: &str) -> ProviderResult<T> {
    Err(unsupported_error(message))
}

fn invalid_error(message: &str) -> ProviderError {
    ProviderError::new(ProviderErrorKind::InvalidContract, message)
}

fn unsupported_error(message: &str) -> ProviderError {
    ProviderError::new(ProviderErrorKind::UnsupportedApplication, message)
}

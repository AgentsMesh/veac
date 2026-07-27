use std::collections::BTreeMap;

use veac_plan::canonical::{ParameterValue, TimeRange};
use veac_plan::{
    ResolvedApply, ResolvedApplyOperation, ResolvedApplyStage, ResolvedClip, ResolvedEffect,
};

use super::error::{diagnostic, CodegenErrorKind};
use super::{animation, process_owner::ProcessOwner, time, CodegenErrors, EmitContext};

mod alpha;
mod dynamic;
mod normalize;

pub(super) use dynamic::{filter as dynamic_filter, RuntimeNumber};

#[derive(Clone, Copy)]
pub(super) struct EffectSpec<'a> {
    pub id: &'a str,
    pub effect_type: &'a str,
    pub active_range: TimeRange,
    pub parameters: &'a BTreeMap<String, ParameterValue>,
}

impl<'a> EffectSpec<'a> {
    fn clip(effect: &'a ResolvedEffect) -> Self {
        Self {
            id: effect.id.as_str(),
            effect_type: &effect.effect_type,
            active_range: effect.active_range,
            parameters: &effect.parameters,
        }
    }

    fn apply(stage: &'a ResolvedApplyStage, active_range: TimeRange) -> Option<Self> {
        let ResolvedApplyOperation::Effect {
            effect_type,
            parameters,
        } = &stage.operation
        else {
            return None;
        };
        Some(Self {
            id: stage.id.as_str(),
            effect_type: effect_type.as_str(),
            active_range,
            parameters,
        })
    }
}

pub(super) fn video(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    mut label: String,
) -> Result<String, CodegenErrors> {
    for effect in &clip.effects {
        if veac_plan::canonical::effect_domain(&effect.effect_type)
            == Some(veac_plan::canonical::EffectDomain::Audio)
        {
            continue;
        }
        let effect = EffectSpec::clip(effect);
        let enable = enable(effect);
        label = alpha::apply(context, effect, &label, |context, input| {
            super::effect_video::apply(context, ProcessOwner::clip(clip), effect, input, &enable)
        })?;
    }
    Ok(label)
}

pub(super) fn apply_stage(
    context: &mut EmitContext<'_>,
    apply: &ResolvedApply,
    stage: &ResolvedApplyStage,
    active_range: TimeRange,
    label: String,
) -> Result<String, CodegenErrors> {
    let effect = EffectSpec::apply(stage, active_range).expect("effect stage requested");
    let enable = enable(effect);
    alpha::apply(context, effect, &label, |context, input| {
        super::effect_video::apply(context, ProcessOwner::apply(apply), effect, input, &enable)
    })
}

pub(super) fn audio(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    mut label: String,
) -> Result<String, CodegenErrors> {
    for effect in &clip.effects {
        if effect.effect_type != "audio.normalize" {
            continue;
        }
        label = normalize::apply(context, clip, effect, &label)?;
    }
    Ok(label)
}

pub(super) fn number_expression(effect: EffectSpec<'_>, name: &str, default: f64) -> String {
    number_expression_at(effect, name, default, "t")
}

pub(super) fn number_expression_at(
    effect: EffectSpec<'_>,
    name: &str,
    default: f64,
    clock: &str,
) -> String {
    match effect.parameters.get(name) {
        Some(ParameterValue::Number { value }) => time::number(*value),
        Some(ParameterValue::NumberCurve { value }) => animation::number(value, clock),
        _ => time::number(default),
    }
}

pub(super) fn has_keyframes(effect: EffectSpec<'_>, name: &str) -> bool {
    matches!(
        effect.parameters.get(name),
        Some(ParameterValue::NumberCurve {
            value: veac_plan::canonical::Animatable::Keyframes { .. }
        })
    )
}

pub(super) fn number(
    effect: &ResolvedEffect,
    name: &str,
    default: f64,
    clip: &ResolvedClip,
) -> Result<String, CodegenErrors> {
    match effect.parameters.get(name) {
        Some(ParameterValue::NumberCurve { .. }) => Err(unsupported_clip(
            clip,
            effect,
            "animated scalar backend parameter",
        )),
        _ => Ok(number_expression(EffectSpec::clip(effect), name, default)),
    }
}

fn enable(effect: EffectSpec<'_>) -> String {
    format!(
        "enable='gte(t,{})*lt(t,{})'",
        time::seconds(effect.active_range.start),
        time::end(effect.active_range)
    )
}

pub(super) fn unsupported(
    owner: ProcessOwner<'_>,
    effect: EffectSpec<'_>,
    detail: &str,
) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::UnsupportedEffect,
        "EFFECT_UNSUPPORTED",
        Some(effect.id.to_owned()),
        format!(
            "{} on {} {} is unsupported: {detail}",
            effect.effect_type,
            owner.kind(),
            owner.id()
        ),
    ))
}

pub(super) fn unsupported_clip(
    clip: &ResolvedClip,
    effect: &ResolvedEffect,
    detail: &str,
) -> CodegenErrors {
    CodegenErrors::one(diagnostic(
        CodegenErrorKind::UnsupportedEffect,
        "EFFECT_UNSUPPORTED",
        Some(effect.id.to_string()),
        format!(
            "{} on clip {} is unsupported: {detail}",
            effect.effect_type, clip.id
        ),
    ))
}

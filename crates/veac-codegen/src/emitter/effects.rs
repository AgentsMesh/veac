use veac_plan::canonical::{Effect, EffectDomain, EffectKind, EffectParameter, TimeRange};
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
    pub active_range: TimeRange,
    pub effect: &'a Effect,
    pub owner: ProcessOwner<'a>,
}

impl<'a> EffectSpec<'a> {
    fn clip(clip: &'a ResolvedClip, effect: &'a ResolvedEffect) -> Self {
        Self {
            id: effect.id.as_str(),
            active_range: effect.active_range,
            effect: &effect.effect,
            owner: ProcessOwner::clip(clip),
        }
    }

    fn apply(
        apply: &'a ResolvedApply,
        stage: &'a ResolvedApplyStage,
        active_range: TimeRange,
    ) -> Option<Self> {
        let ResolvedApplyOperation::Effect { effect } = &stage.operation else {
            return None;
        };
        Some(Self {
            id: stage.id.as_str(),
            active_range,
            effect,
            owner: ProcessOwner::apply(apply),
        })
    }
}

pub(super) fn video(
    context: &mut EmitContext<'_>,
    clip: &ResolvedClip,
    mut label: String,
) -> Result<String, CodegenErrors> {
    for effect in &clip.effects {
        if effect.effect.domain() == EffectDomain::Audio {
            continue;
        }
        let effect = EffectSpec::clip(clip, effect);
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
    let effect = EffectSpec::apply(apply, stage, active_range).expect("effect stage requested");
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
        if effect.effect.kind() != EffectKind::AudioNormalize {
            continue;
        }
        label = normalize::apply(context, clip, effect, &label)?;
    }
    Ok(label)
}

pub(super) fn number_expression(
    context: &EmitContext<'_>,
    effect: EffectSpec<'_>,
    parameter: EffectParameter,
    default: f64,
) -> String {
    number_expression_at(context, effect, parameter, default, "t")
}

pub(super) fn number_expression_at(
    context: &EmitContext<'_>,
    effect: EffectSpec<'_>,
    parameter: EffectParameter,
    default: f64,
    clock: &str,
) -> String {
    match effect.effect.curve(parameter) {
        Some(value) => animation::number(context.plan, effect.owner, value, clock),
        _ => time::number(default),
    }
}

pub(super) fn has_keyframes(effect: EffectSpec<'_>, parameter: EffectParameter) -> bool {
    matches!(
        effect.effect.curve(parameter),
        Some(
            veac_plan::canonical::Animatable::Keyframes { .. }
                | veac_plan::canonical::Animatable::Binding { .. }
        )
    )
}

pub(super) fn number(
    effect: &ResolvedEffect,
    parameter: EffectParameter,
    default: f64,
    clip: &ResolvedClip,
) -> Result<String, CodegenErrors> {
    if effect.effect.curve(parameter).is_some() {
        return Err(unsupported_clip(
            clip,
            effect,
            "animated scalar backend parameter",
        ));
    }
    Ok(time::number(
        effect.effect.number(parameter).unwrap_or(default),
    ))
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
            effect.effect.kind().type_name(),
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
            effect.effect.kind().type_name(),
            clip.id
        ),
    ))
}

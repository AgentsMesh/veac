use crate::authoring::{AudioFadeCurveDecl, AudioModifierDecl, NumberLiteral, PitchPolicyDecl};
use veac_ir::{Animatable, AudioCrossfade, AudioFadeCurve, AudioProperties, PitchPolicy};

use super::context::Context;
use super::{animation, audio_processor, value};

pub(super) fn lower(ctx: &mut Context, value: &AudioModifierDecl) -> Option<AudioProperties> {
    Some(AudioProperties {
        gain: match &value.gain {
            Some(value) => animation::parameter(ctx, value, gain)?,
            None => Animatable::constant(1.0),
        },
        pan: match &value.pan {
            Some(value) => animation::parameter(ctx, value, value::unitless)?,
            None => Animatable::constant(0.0),
        },
        muted: value.muted.as_ref().is_some_and(|value| value.value),
        normalize: value.normalize.as_ref().is_some_and(|value| value.value),
        pitch_policy: match value.pitch.as_ref().map(|value| value.value) {
            Some(PitchPolicyDecl::FollowSpeed) => PitchPolicy::FollowSpeed,
            Some(PitchPolicyDecl::Preserve) | None => PitchPolicy::Preserve,
        },
        processors: value
            .processors
            .iter()
            .map(|value| audio_processor::lower(ctx, value))
            .collect::<Option<Vec<_>>>()?,
        crossfade: match &value.crossfade {
            Some(value) => Some(AudioCrossfade {
                fade_in: super::value::time(ctx, &value.fade_in)?,
                fade_out: super::value::time(ctx, &value.fade_out)?,
                curve: match value.curve.value {
                    AudioFadeCurveDecl::Linear => AudioFadeCurve::Linear,
                    AudioFadeCurveDecl::EqualPower => AudioFadeCurve::EqualPower,
                    AudioFadeCurveDecl::Exponential => AudioFadeCurve::Exponential,
                },
            }),
            None => None,
        },
    })
}

fn gain(ctx: &mut Context, value: &NumberLiteral) -> Option<f64> {
    let (_, unit) = super::value::split(&value.raw);
    match unit {
        "db" => Some(10_f64.powf(super::value::scalar(ctx, value, "db")? / 20.0)),
        "" | "%" => value::scale(ctx, value),
        _ => {
            ctx.error(
                "AUTHORING_LOWER_AUDIO_GAIN",
                "gain must be in db, percent, or unitless linear amplitude",
                value.span,
            );
            None
        }
    }
}

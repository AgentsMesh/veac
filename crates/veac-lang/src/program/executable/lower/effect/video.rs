use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{Animatable, Effect, PluginEffectDigest, REFERENCE_MONOCHROME_DIGEST};

use super::{curve_scope, malformed, percent_curve, scalar_curve, Context, ExecutableLowerError};
use crate::program::executable::lower::{animation, value};

pub(super) fn lower(
    context: &Context<'_>,
    operation: Op,
    values: &[Value],
    key: &str,
) -> Result<Effect, ExecutableLowerError> {
    match (operation, values) {
        (Op::VideoColorAdjustEffect, [_, _, brightness, contrast, saturation]) => {
            Ok(Effect::VideoColorAdjust {
                brightness: scalar(context, key, "brightness", brightness)?,
                contrast: scalar(context, key, "contrast", contrast)?,
                saturation: scalar(context, key, "saturation", saturation)?,
            })
        }
        (Op::VideoBlurEffect, [_, _, radius]) => Ok(Effect::VideoBlur {
            radius: length(context, key, radius)?,
        }),
        (Op::VideoDirectionalBlurEffect, [_, _, angle_source, radius]) => {
            Ok(Effect::VideoDirectionalBlur {
                angle_degrees: angle(context, key, angle_source)?,
                radius: length(context, key, radius)?,
            })
        }
        (Op::VideoSharpenEffect, [_, _, amount]) => Ok(Effect::VideoSharpen {
            amount: scalar(context, key, "amount", amount)?,
        }),
        (Op::VideoVignetteEffect, [_, _, amount]) => Ok(Effect::VideoVignette {
            amount: percent(context, key, "amount", amount)?,
        }),
        (Op::VideoGrainEffect, [_, _, amount]) => Ok(Effect::VideoGrain {
            amount: percent(context, key, "amount", amount)?,
        }),
        (Op::VideoChromaKeyEffect, [_, _, color, similarity, blend]) => {
            Ok(Effect::VideoChromaKey {
                color: value::color(Some(color))?,
                similarity: percent(context, key, "similarity", similarity)?,
                blend: percent(context, key, "blend", blend)?,
            })
        }
        (Op::VideoLumaKeyEffect, [_, _, threshold, tolerance, softness, invert]) => {
            Ok(Effect::VideoLumaKey {
                threshold: percent(context, key, "threshold", threshold)?,
                tolerance: percent(context, key, "tolerance", tolerance)?,
                softness: percent(context, key, "softness", softness)?,
                invert: value::boolean(Some(invert))?,
            })
        }
        (Op::VideoChromaSpillEffect, [_, _, color, amount, range]) => {
            Ok(Effect::VideoChromaSpill {
                color: value::color(Some(color))?,
                amount: percent(context, key, "amount", amount)?,
                range: percent(context, key, "range", range)?,
            })
        }
        (Op::VideoStabilizeEffect, [_, _, enabled]) => Ok(Effect::VideoStabilize {
            enabled: value::boolean(Some(enabled))?,
        }),
        (Op::AudioNormalizeEffect, [_, _, target]) => Ok(Effect::AudioNormalize {
            target_lufs: value::finite(Some(target))?,
        }),
        (Op::VideoPluginScalarEffect, [_, _, descriptor, amount]) => {
            plugin(context, descriptor, amount, key)
        }
        _ => Err(malformed()),
    }
}

fn plugin(
    context: &Context<'_>,
    descriptor: &Value,
    amount: &Value,
    key: &str,
) -> Result<Effect, ExecutableLowerError> {
    let (operation, values) = value::description(context.graph, descriptor)?;
    if operation != Op::PluginReferenceMonochromeV1 || !values.is_empty() {
        return Err(malformed());
    }
    Ok(Effect::VideoPluginReferenceMonochromeV1 {
        descriptor_digest: PluginEffectDigest::new(REFERENCE_MONOCHROME_DIGEST)
            .map_err(|_| malformed())?,
        amount: scalar(context, key, "amount", amount)?,
    })
}

fn scalar(
    context: &Context<'_>,
    key: &str,
    name: &str,
    source: &Value,
) -> Result<Animatable<f64>, ExecutableLowerError> {
    scalar_curve(context, source, &curve_scope(context, key, name))
}

fn percent(
    context: &Context<'_>,
    key: &str,
    name: &str,
    source: &Value,
) -> Result<Animatable<f64>, ExecutableLowerError> {
    percent_curve(context, source, &curve_scope(context, key, name))
}

fn angle(
    context: &Context<'_>,
    key: &str,
    source: &Value,
) -> Result<Animatable<f64>, ExecutableLowerError> {
    animation::angle(
        context.graph,
        source,
        context.timebase,
        &curve_scope(context, key, "angle_degrees"),
    )
}

fn length(
    context: &Context<'_>,
    key: &str,
    source: &Value,
) -> Result<Animatable<f64>, ExecutableLowerError> {
    let scope = curve_scope(context, key, "radius");
    let curve = animation::length(context.graph, source, context.timebase, &scope)?;
    Ok(length_pixels(curve))
}

fn length_pixels(value: Animatable<veac_ir::Length>) -> Animatable<f64> {
    match value {
        Animatable::Constant { value } => Animatable::constant(value.value),
        Animatable::Keyframes { keyframes } => Animatable::Keyframes {
            keyframes: keyframes
                .into_iter()
                .map(|key| veac_ir::Keyframe {
                    id: key.id,
                    time: key.time,
                    value: key.value.value,
                    interpolation: key.interpolation,
                })
                .collect(),
        },
        Animatable::Binding { binding_id } => Animatable::Binding { binding_id },
    }
}

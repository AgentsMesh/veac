use crate::authoring::{InterpolationDecl, InterpolationKind, NumberLiteral, ParameterDecl};
use veac_ir::{Animatable, Interpolation, Keyframe};

use super::{context::Context, ids, value};

pub fn parameter<T, U>(
    ctx: &mut Context,
    parameter: &ParameterDecl<T>,
    mut convert: impl FnMut(&mut Context, &T) -> Option<U>,
) -> Option<Animatable<U>> {
    match parameter {
        ParameterDecl::Constant(value) => Some(Animatable::constant(convert(ctx, value)?)),
        ParameterDecl::Curve { keys, .. } => {
            let mut keyframes = Vec::with_capacity(keys.len());
            for key in keys {
                keyframes.push(Keyframe {
                    id: ids::key(ctx, &key.id)?,
                    time: value::time(ctx, &key.at)?,
                    value: convert(ctx, &key.value)?,
                    interpolation: interpolation(ctx, &key.interpolation)?,
                });
            }
            Some(Animatable::Keyframes { keyframes })
        }
    }
}

pub fn scalar(
    ctx: &mut Context,
    declaration: &ParameterDecl<NumberLiteral>,
    unit: &str,
) -> Option<Animatable<f64>> {
    parameter(ctx, declaration, |ctx, value| {
        value::scalar(ctx, value, unit)
    })
}

fn interpolation(ctx: &mut Context, value: &InterpolationDecl) -> Option<Interpolation> {
    let result = match &value.kind {
        InterpolationKind::Hold => Interpolation::Hold,
        InterpolationKind::Linear => Interpolation::Linear,
        InterpolationKind::EaseIn => Interpolation::EaseIn,
        InterpolationKind::EaseOut => Interpolation::EaseOut,
        InterpolationKind::EaseInOut => Interpolation::EaseInOut,
        InterpolationKind::Spring {
            frequency,
            decay,
            initial_velocity,
        } => Interpolation::Spring {
            frequency: value::unitless(ctx, frequency)?,
            decay: value::unitless(ctx, decay)?,
            initial_velocity: value::unitless(ctx, initial_velocity)?,
        },
        InterpolationKind::CubicBezier { x1, y1, x2, y2 } => Interpolation::CubicBezier {
            x1: value::unitless(ctx, x1)?,
            y1: value::unitless(ctx, y1)?,
            x2: value::unitless(ctx, x2)?,
            y2: value::unitless(ctx, y2)?,
        },
    };
    Some(result)
}

use veac_plan::canonical::{Animatable, Keyframe, Point, TemporalValue, Vec2};
use veac_plan::{ResolvedClip, ResolvedRenderPlan};

use crate::emitter::{geometry, process_owner::ProcessOwner, temporal, text::error::TextError};

pub(super) fn constant<T>(value: &Animatable<T>) -> bool {
    matches!(value, Animatable::Constant { .. })
}

pub(super) fn number(value: &Animatable<f64>, seconds: f64) -> f64 {
    match value {
        Animatable::Constant { value } => *value,
        Animatable::Keyframes { keyframes } => match locate(keyframes, seconds) {
            Location::Empty => 0.0,
            Location::Value(value) => *value,
            Location::Between(left, right, amount) => left + (right - left) * amount,
        },
        Animatable::Binding { .. } => {
            unreachable!("temporal bindings are rejected by codegen preflight")
        }
    }
}

pub(super) fn bound_number(
    plan: &ResolvedRenderPlan,
    clip: &ResolvedClip,
    value: &Animatable<f64>,
    seconds: f64,
) -> Result<f64, TextError> {
    let Animatable::Binding { binding_id } = value else {
        return Ok(number(value, seconds));
    };
    match evaluate(plan, clip, binding_id, seconds)? {
        TemporalValue::Scalar { value } => Ok(value),
        TemporalValue::Angle { degrees } => Ok(degrees),
        _ => Err(TextError::invalid(
            "temporal text number has the wrong type",
        )),
    }
}

pub(super) fn point(value: &Animatable<Point>, seconds: f64, surface: (u32, u32)) -> (f64, f64) {
    let resolve = |value: &Point| {
        (
            geometry::pixel_value(value.x, surface.0),
            geometry::pixel_value(value.y, surface.1),
        )
    };
    match value {
        Animatable::Constant { value } => resolve(value),
        Animatable::Keyframes { keyframes } => match locate(keyframes, seconds) {
            Location::Empty => (0.0, 0.0),
            Location::Value(value) => resolve(value),
            Location::Between(left, right, amount) => pair(resolve(left), resolve(right), amount),
        },
        Animatable::Binding { .. } => {
            unreachable!("temporal bindings are rejected by codegen preflight")
        }
    }
}

pub(super) fn bound_point(
    plan: &ResolvedRenderPlan,
    clip: &ResolvedClip,
    value: &Animatable<Point>,
    seconds: f64,
    surface: (u32, u32),
) -> Result<(f64, f64), TextError> {
    let Animatable::Binding { binding_id } = value else {
        return Ok(point(value, seconds, surface));
    };
    match evaluate(plan, clip, binding_id, seconds)? {
        TemporalValue::Point { value } => Ok((
            geometry::pixel_value(value.x, surface.0),
            geometry::pixel_value(value.y, surface.1),
        )),
        _ => Err(TextError::invalid("temporal text point has the wrong type")),
    }
}

pub(super) fn vec2(value: &Animatable<Vec2>, seconds: f64) -> (f64, f64) {
    let resolve = |value: &Vec2| (value.x, value.y);
    match value {
        Animatable::Constant { value } => resolve(value),
        Animatable::Keyframes { keyframes } => match locate(keyframes, seconds) {
            Location::Empty => (1.0, 1.0),
            Location::Value(value) => resolve(value),
            Location::Between(left, right, amount) => pair(resolve(left), resolve(right), amount),
        },
        Animatable::Binding { .. } => {
            unreachable!("temporal bindings are rejected by codegen preflight")
        }
    }
}

pub(super) fn bound_vec2(
    plan: &ResolvedRenderPlan,
    clip: &ResolvedClip,
    value: &Animatable<Vec2>,
    seconds: f64,
) -> Result<(f64, f64), TextError> {
    let Animatable::Binding { binding_id } = value else {
        return Ok(vec2(value, seconds));
    };
    match evaluate(plan, clip, binding_id, seconds)? {
        TemporalValue::Vec2 { value } => Ok((value.x, value.y)),
        _ => Err(TextError::invalid(
            "temporal text vector has the wrong type",
        )),
    }
}

fn evaluate(
    plan: &ResolvedRenderPlan,
    clip: &ResolvedClip,
    binding_id: &veac_plan::canonical::TemporalBindingId,
    seconds: f64,
) -> Result<TemporalValue, TextError> {
    temporal::evaluate_binding(plan, binding_id, ProcessOwner::clip(clip), seconds)
        .map_err(|error| TextError::new(error.code, error.message))
}

enum Location<'a, T> {
    Empty,
    Value(&'a T),
    Between(&'a T, &'a T, f64),
}

fn locate<T>(keyframes: &[Keyframe<T>], seconds: f64) -> Location<'_, T> {
    let Some(first) = keyframes.first() else {
        return Location::Empty;
    };
    if seconds <= time(first.time) {
        return Location::Value(&first.value);
    }
    let Some(pair) = keyframes
        .windows(2)
        .find(|pair| seconds <= time(pair[1].time))
    else {
        return keyframes
            .last()
            .map_or(Location::Empty, |key| Location::Value(&key.value));
    };
    let start = time(pair[0].time);
    let end = time(pair[1].time);
    let progress = ((seconds - start) / (end - start)).clamp(0.0, 1.0);
    let amount = pair[0].interpolation.evaluate(progress);
    Location::Between(&pair[0].value, &pair[1].value, amount)
}

fn pair(left: (f64, f64), right: (f64, f64), amount: f64) -> (f64, f64) {
    (
        left.0 + (right.0 - left.0) * amount,
        left.1 + (right.1 - left.1) * amount,
    )
}

fn time(value: veac_plan::canonical::RationalTime) -> f64 {
    value.value as f64 / f64::from(value.timescale)
}

#[cfg(test)]
mod tests;

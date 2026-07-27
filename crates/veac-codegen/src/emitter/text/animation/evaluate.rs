use veac_plan::canonical::{Animatable, Keyframe, Point, Vec2};

use crate::emitter::geometry;

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
    }
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

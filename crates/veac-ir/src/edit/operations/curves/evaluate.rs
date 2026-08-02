use crate::*;

#[cfg(test)]
mod tests;

pub(super) trait CurveValue: Clone {
    fn interpolate(left: &Self, right: &Self, amount: f64) -> Option<Self>;
}

impl CurveValue for f64 {
    fn interpolate(left: &Self, right: &Self, amount: f64) -> Option<Self> {
        Some(left + (right - left) * amount)
    }
}

impl CurveValue for Vec2 {
    fn interpolate(left: &Self, right: &Self, amount: f64) -> Option<Self> {
        Some(Self {
            x: left.x + (right.x - left.x) * amount,
            y: left.y + (right.y - left.y) * amount,
        })
    }
}

impl CurveValue for Rect {
    fn interpolate(left: &Self, right: &Self, amount: f64) -> Option<Self> {
        Some(Self {
            x: left.x + (right.x - left.x) * amount,
            y: left.y + (right.y - left.y) * amount,
            width: left.width + (right.width - left.width) * amount,
            height: left.height + (right.height - left.height) * amount,
        })
    }
}

impl CurveValue for Point {
    fn interpolate(left: &Self, right: &Self, amount: f64) -> Option<Self> {
        if left.x.unit != right.x.unit || left.y.unit != right.y.unit {
            return None;
        }
        Some(Self {
            x: Length {
                value: left.x.value + (right.x.value - left.x.value) * amount,
                unit: left.x.unit,
            },
            y: Length {
                value: left.y.value + (right.y.value - left.y.value) * amount,
                unit: left.y.unit,
            },
        })
    }
}

pub(super) fn at<T: CurveValue>(keys: &[Keyframe<T>], time: RationalTime) -> Option<T> {
    let first = keys.first()?;
    if time <= first.time {
        return Some(first.value.clone());
    }
    let Some(pair) = keys
        .windows(2)
        .find(|pair| time >= pair[0].time && time <= pair[1].time)
    else {
        return keys.last().map(|key| key.value.clone());
    };
    if time == pair[1].time {
        return Some(pair[1].value.clone());
    }
    let span = pair[1].time.value - pair[0].time.value;
    let offset = time.value - pair[0].time.value;
    let amount = pair[0].interpolation.evaluate(offset as f64 / span as f64);
    T::interpolate(&pair[0].value, &pair[1].value, amount)
}

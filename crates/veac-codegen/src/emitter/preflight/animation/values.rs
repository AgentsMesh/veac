use veac_plan::canonical::{Interpolation, Keyframe, Length, Point, Rect, Vec2};

pub(super) trait SpringValue: Sized {
    fn spring_value(&self, next: &Self, amount: f64) -> Option<Self>;
}

pub(super) fn spring_ranges_valid<T: SpringValue>(
    keyframes: &[Keyframe<T>],
    valid: fn(&T) -> bool,
) -> bool {
    keyframes.windows(2).all(|pair| {
        if !matches!(pair[0].interpolation, Interpolation::Spring { .. }) {
            return true;
        }
        let Some(amounts) = pair[0].interpolation.spring_extrema() else {
            return false;
        };
        amounts.into_iter().all(|amount| {
            pair[0]
                .value
                .spring_value(&pair[1].value, amount)
                .is_some_and(|value| valid(&value))
        })
    })
}

pub(super) fn finite(value: &f64) -> bool {
    value.is_finite()
}

pub(super) fn nonnegative(value: &f64) -> bool {
    value.is_finite() && *value >= 0.0
}

pub(super) fn unit_number(value: &f64) -> bool {
    value.is_finite() && (0.0..=1.0).contains(value)
}

pub(super) fn pan(value: &f64) -> bool {
    value.is_finite() && (-1.0..=1.0).contains(value)
}

pub(super) fn point(value: &Point) -> bool {
    value.x.value.is_finite() && value.y.value.is_finite()
}

pub(super) fn positive_vec(value: &Vec2) -> bool {
    veac_plan::canonical::visual_scale_valid(*value)
}

pub(super) fn unit_vec(value: &Vec2) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && (0.0..=1.0).contains(&value.x)
        && (0.0..=1.0).contains(&value.y)
}

pub(super) fn rect(value: &Rect) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && value.width.is_finite()
        && value.height.is_finite()
        && value.x >= 0.0
        && value.y >= 0.0
        && value.width >= veac_plan::canonical::MIN_CROP_EXTENT
        && value.height >= veac_plan::canonical::MIN_CROP_EXTENT
        && value.x + value.width <= 1.0
        && value.y + value.height <= 1.0
}

pub(super) fn interpolation(value: &Interpolation) -> bool {
    match value {
        Interpolation::CubicBezier { x1, y1, x2, y2 } => {
            x1.is_finite()
                && y1.is_finite()
                && x2.is_finite()
                && y2.is_finite()
                && (0.0..=1.0).contains(x1)
                && (0.0..=1.0).contains(y1)
                && (0.0..=1.0).contains(x2)
                && (0.0..=1.0).contains(y2)
        }
        Interpolation::Spring { .. } => value.spring_coefficients().is_some(),
        _ => true,
    }
}

impl SpringValue for f64 {
    fn spring_value(&self, next: &Self, amount: f64) -> Option<Self> {
        Some(self + (next - self) * amount)
    }
}

impl SpringValue for Vec2 {
    fn spring_value(&self, next: &Self, amount: f64) -> Option<Self> {
        Some(Self {
            x: self.x + (next.x - self.x) * amount,
            y: self.y + (next.y - self.y) * amount,
        })
    }
}

impl SpringValue for Point {
    fn spring_value(&self, next: &Self, amount: f64) -> Option<Self> {
        if self.x.unit != next.x.unit || self.y.unit != next.y.unit {
            return None;
        }
        Some(Self {
            x: Length {
                value: self.x.value + (next.x.value - self.x.value) * amount,
                unit: self.x.unit,
            },
            y: Length {
                value: self.y.value + (next.y.value - self.y.value) * amount,
                unit: self.y.unit,
            },
        })
    }
}

impl SpringValue for Rect {
    fn spring_value(&self, next: &Self, amount: f64) -> Option<Self> {
        Some(Self {
            x: self.x + (next.x - self.x) * amount,
            y: self.y + (next.y - self.y) * amount,
            width: self.width + (next.width - self.width) * amount,
            height: self.height + (next.height - self.height) * amount,
        })
    }
}

#[cfg(test)]
#[path = "values/tests.rs"]
mod tests;

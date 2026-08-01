use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn animatable<T: SpringSample>(
        &mut self,
        value: &Animatable<T>,
        duration: RationalTime,
        timebase: u32,
        path: &str,
        item_id: &str,
        valid_value: impl Fn(&T) -> bool,
    ) {
        match value {
            Animatable::Constant { value } => {
                if !valid_value(value) {
                    self.value_error("ANIMATION_VALUE", path, item_id);
                }
            }
            Animatable::Keyframes { keyframes } => {
                if keyframes.is_empty() {
                    self.value_error("EMPTY_KEYFRAMES", path, item_id);
                }
                for keyframe in keyframes {
                    self.keyframe(keyframe, timebase, path, item_id, &valid_value);
                }
                self.keyframe_order(keyframes, duration, path, item_id);
                for pair in keyframes.windows(2) {
                    let Some(amounts) = pair[0].interpolation.spring_extrema() else {
                        continue;
                    };
                    for amount in amounts {
                        let Some(value) = pair[0].value.spring_sample(&pair[1].value, amount)
                        else {
                            self.value_error("ANIMATION_VALUE", path, item_id);
                            continue;
                        };
                        if !valid_value(&value) {
                            self.value_error("ANIMATION_VALUE", path, item_id);
                        }
                    }
                }
            }
        }
    }

    fn keyframe<T>(
        &mut self,
        keyframe: &Keyframe<T>,
        timebase: u32,
        path: &str,
        item_id: &str,
        valid_value: &impl Fn(&T) -> bool,
    ) {
        self.check_id(keyframe.id.is_valid(), keyframe.id.as_str(), path);
        if !self.keyframe_ids.insert(keyframe.id.to_string()) {
            self.duplicate("DUPLICATE_KEYFRAME_ID", keyframe.id.as_str(), path);
        }
        self.time(
            keyframe.time,
            timebase,
            false,
            "KEYFRAME_TIME",
            path,
            item_id,
        );
        if !valid_value(&keyframe.value) {
            self.value_error("ANIMATION_VALUE", path, item_id);
        }
        match &keyframe.interpolation {
            Interpolation::CubicBezier { x1, y1, x2, y2 } => {
                let valid = x1.is_finite()
                    && y1.is_finite()
                    && x2.is_finite()
                    && y2.is_finite()
                    && (0.0..=1.0).contains(x1)
                    && (0.0..=1.0).contains(y1)
                    && (0.0..=1.0).contains(x2)
                    && (0.0..=1.0).contains(y2);
                if !valid {
                    self.value_error("BEZIER", path, item_id);
                }
            }
            Interpolation::Spring { .. }
                if keyframe.interpolation.spring_coefficients().is_none() =>
            {
                self.value_error("SPRING", path, item_id);
            }
            _ => {}
        }
    }

    fn keyframe_order<T>(
        &mut self,
        keyframes: &[Keyframe<T>],
        duration: RationalTime,
        path: &str,
        item_id: &str,
    ) {
        if keyframes.iter().any(|keyframe| keyframe.time > duration)
            || keyframes
                .windows(2)
                .any(|pair| pair[1].time <= pair[0].time)
        {
            self.value_error("KEYFRAME_ORDER", path, item_id);
        }
    }
}

pub(super) trait SpringSample: Sized {
    fn spring_sample(&self, next: &Self, amount: f64) -> Option<Self>;
}

impl SpringSample for f64 {
    fn spring_sample(&self, next: &Self, amount: f64) -> Option<Self> {
        Some(self + (next - self) * amount)
    }
}

impl SpringSample for Vec2 {
    fn spring_sample(&self, next: &Self, amount: f64) -> Option<Self> {
        Some(Self {
            x: self.x + (next.x - self.x) * amount,
            y: self.y + (next.y - self.y) * amount,
        })
    }
}

impl SpringSample for Point {
    fn spring_sample(&self, next: &Self, amount: f64) -> Option<Self> {
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

impl SpringSample for Rect {
    fn spring_sample(&self, next: &Self, amount: f64) -> Option<Self> {
        Some(Self {
            x: self.x + (next.x - self.x) * amount,
            y: self.y + (next.y - self.y) * amount,
            width: self.width + (next.width - self.width) * amount,
            height: self.height + (next.height - self.height) * amount,
        })
    }
}

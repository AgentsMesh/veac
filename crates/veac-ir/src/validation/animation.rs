use crate::*;

use super::Validator;

impl Validator {
    pub(super) fn animatable<T>(
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
        if let Interpolation::CubicBezier { x1, y1, x2, y2 } = keyframe.interpolation {
            let valid = x1.is_finite()
                && y1.is_finite()
                && x2.is_finite()
                && y2.is_finite()
                && (0.0..=1.0).contains(&x1)
                && (0.0..=1.0).contains(&y1)
                && (0.0..=1.0).contains(&x2)
                && (0.0..=1.0).contains(&y2);
            if !valid {
                self.value_error("BEZIER", path, item_id);
            }
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

use veac_plan::canonical::{Interpolation, Keyframe, RationalTime};

pub(super) type KeyMutation = fn(&mut [Keyframe<f64>], RationalTime);

pub(super) fn invalid_id(keys: &mut [Keyframe<f64>], _: RationalTime) {
    keys[0].id = serde_json::from_str("\"invalid\"").unwrap();
}

pub(super) fn zero_timebase(keys: &mut [Keyframe<f64>], _: RationalTime) {
    keys[0].time.timescale = 0;
}

pub(super) fn negative_time(keys: &mut [Keyframe<f64>], _: RationalTime) {
    keys[0].time.value = -1;
}

pub(super) fn foreign_timebase(keys: &mut [Keyframe<f64>], _: RationalTime) {
    keys[0].time.timescale = 30;
}

pub(super) fn equal_time(keys: &mut [Keyframe<f64>], _: RationalTime) {
    keys[1].time = keys[0].time;
}

pub(super) fn descending_time(keys: &mut [Keyframe<f64>], _: RationalTime) {
    keys[0].time.value = 300;
    keys[1].time.value = 100;
}

pub(super) fn outside_clip(keys: &mut [Keyframe<f64>], duration: RationalTime) {
    keys[1].time.value = duration.value + 1;
}

pub(super) fn unsafe_time(keys: &mut [Keyframe<f64>], _: RationalTime) {
    keys[1].time.value = i64::MAX;
}

pub(super) fn invalid_easing(keys: &mut [Keyframe<f64>], _: RationalTime) {
    keys[0].interpolation = Interpolation::CubicBezier {
        x1: -0.1,
        y1: 0.1,
        x2: 1.0,
        y2: 1.0,
    };
}

pub(super) fn invalid_value(keys: &mut [Keyframe<f64>], _: RationalTime) {
    keys[0].value = f64::NAN;
}

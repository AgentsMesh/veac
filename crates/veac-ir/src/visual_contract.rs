/// Practical blur budget for FFmpeg shadow expansion and Gaussian filtering.
pub const MAX_SHADOW_BLUR_PIXELS: f64 = 256.0;
pub const MAX_VISUAL_SCALE: f64 = 16.0;
pub const MIN_CROP_EXTENT: f64 = 1.0 / 64.0;

pub fn shadow_blur_valid(value: f64) -> bool {
    value.is_finite() && (0.0..=MAX_SHADOW_BLUR_PIXELS).contains(&value)
}

pub fn visual_scale_valid(value: crate::Vec2) -> bool {
    value.x.is_finite()
        && value.y.is_finite()
        && (0.0..=MAX_VISUAL_SCALE).contains(&value.x)
        && (0.0..=MAX_VISUAL_SCALE).contains(&value.y)
        && value.x > 0.0
        && value.y > 0.0
}

pub const MAX_TEXT_BYTES: usize = 65_536;
pub const MAX_TEXT_SCALARS: usize = 65_536;
pub const MAX_TEXT_SPANS: usize = 4_096;
pub const MAX_FALLBACK_FONTS: usize = 32;
pub const MAX_TEXT_SIZE_PIXELS: f64 = 4_096.0;
pub const MAX_TEXT_TRACKING_PIXELS: f64 = 4_096.0;
pub const MAX_TEXT_LINE_HEIGHT: f64 = 16.0;
pub const MAX_TEXT_BOX_DIMENSION: f64 = 8_192.0;
pub const MAX_TEXT_BOX_PIXELS: f64 = 16_777_216.0;
pub const MAX_TEXT_PADDING_PIXELS: f64 = 4_096.0;
pub const MAX_TEXT_OUTLINE_PIXELS: f64 = 256.0;

pub fn text_box_valid(width: Option<f64>, height: Option<f64>) -> bool {
    let axis_valid = |value: Option<f64>| {
        value
            .is_none_or(|value| value.is_finite() && value > 0.0 && value <= MAX_TEXT_BOX_DIMENSION)
    };
    axis_valid(width)
        && axis_valid(height)
        && width
            .zip(height)
            .is_none_or(|(width, height)| width * height <= MAX_TEXT_BOX_PIXELS)
}

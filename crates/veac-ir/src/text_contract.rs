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
    match (width, height) {
        (None, None) => true,
        (Some(width), Some(height)) => {
            width.is_finite()
                && height.is_finite()
                && width > 0.0
                && height > 0.0
                && width <= MAX_TEXT_BOX_DIMENSION
                && height <= MAX_TEXT_BOX_DIMENSION
                && width * height <= MAX_TEXT_BOX_PIXELS
        }
        _ => false,
    }
}

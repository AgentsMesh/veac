use super::arithmetic::ceil_div;
use crate::MAX_VISUAL_SCALE;

const SCALE_UNITS: u128 = 1_000_000;

pub fn visual_transform_pixels(
    width: u32,
    height: u32,
    scale_x: f64,
    scale_y: f64,
    allocates_rotation_square: bool,
) -> Option<u128> {
    if width == 0 || height == 0 {
        return None;
    }
    let width = scaled_extent(width, scale_x)?;
    let height = scaled_extent(height, scale_y)?;
    if !allocates_rotation_square {
        return Some(width.saturating_mul(height));
    }
    // The backend pads around an arbitrary pivot by at most 2x per axis, then uses
    // ceil(hypot(width, height)) for both output dimensions.
    let width = width.saturating_mul(2);
    let height = height.saturating_mul(2);
    let squared = width
        .saturating_mul(width)
        .saturating_add(height.saturating_mul(height));
    Some(
        squared
            .saturating_add(width.saturating_add(height).saturating_mul(2))
            .saturating_add(1),
    )
}

pub fn shadow_intermediate_pixels(width: u32, height: u32, blur: f64) -> Option<u128> {
    if width == 0 || height == 0 || !blur.is_finite() || blur < 0.0 {
        return None;
    }
    let half_pad = (blur * 2.0).ceil();
    if half_pad > u64::MAX as f64 {
        return None;
    }
    let total_pad = (half_pad as u128).saturating_mul(2);
    Some(
        u128::from(width)
            .saturating_add(total_pad)
            .saturating_mul(u128::from(height).saturating_add(total_pad)),
    )
}

fn scaled_extent(extent: u32, scale: f64) -> Option<u128> {
    if !scale.is_finite() || scale <= 0.0 || scale > MAX_VISUAL_SCALE {
        return None;
    }
    let fixed = (scale * SCALE_UNITS as f64).ceil();
    if !(1.0..=u64::MAX as f64).contains(&fixed) {
        return None;
    }
    Some(ceil_div(
        u128::from(extent).saturating_mul(fixed as u128),
        SCALE_UNITS,
    ))
}

#[cfg(test)]
#[path = "visual_tests.rs"]
mod tests;

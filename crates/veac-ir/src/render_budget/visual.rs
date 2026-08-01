use super::arithmetic::ceil_div;
use crate::{FitMode, Vec2, MAX_VISUAL_SCALE, MAX_VISUAL_SHEAR};

#[path = "visual/math.rs"]
mod math;
#[path = "visual/placement.rs"]
mod placement;
#[path = "visual/scale.rs"]
mod scale;

use math::ceil_sqrt;
pub use placement::*;
pub use scale::*;

const SCALE_UNITS: u128 = 1_000_000;

pub fn visual_intermediate_pixels(
    transformed: (u128, u128),
    placement: Option<(u128, u128)>,
    canvas: (u32, u32),
    shadow_blur: Option<f64>,
) -> Option<u128> {
    let transformed = transformed.0.checked_mul(transformed.1)?;
    let placement = match placement {
        Some((width, height)) => width.checked_mul(height)?,
        None => 0,
    };
    let shadow = match shadow_blur {
        Some(blur) => shadow_intermediate_pixels(canvas.0, canvas.1, blur)?,
        None => 0,
    };
    Some(transformed.max(placement).max(shadow))
}

pub fn visual_transform_extent(
    width: u32,
    height: u32,
    scale: Vec2,
    pivot_off_center: bool,
    shear: Vec2,
    allocates_rotation_square: bool,
) -> Option<(u128, u128)> {
    if width == 0 || height == 0 {
        return None;
    }
    let mut width = scaled_extent(width, scale.x)?;
    let mut height = scaled_extent(height, scale.y)?;
    let shear_active = shear.x != 0.0 || shear.y != 0.0;
    if pivot_off_center && shear_active {
        width = width.saturating_mul(2);
        height = height.saturating_mul(2);
    }
    let sheared_width = width.saturating_add(shear_extent(height, shear.x)?);
    let sheared_height = height.saturating_add(shear_extent(width, shear.y)?);
    width = sheared_width;
    height = sheared_height;
    if pivot_off_center && allocates_rotation_square {
        width = width.saturating_mul(2);
        height = height.saturating_mul(2);
    }
    if !allocates_rotation_square {
        return Some((width, height));
    }
    let squared = width
        .saturating_mul(width)
        .saturating_add(height.saturating_mul(height));
    let side = ceil_sqrt(squared);
    Some((side, side))
}

pub fn visible_overflow_frame_extent(
    surface: (u32, u32),
    basis: (f64, f64),
    target: (u32, u32),
    fit: FitMode,
) -> Option<(u32, u32)> {
    if surface.0 == 0
        || surface.1 == 0
        || !basis.0.is_finite()
        || !basis.1.is_finite()
        || basis.0 <= 0.0
        || basis.1 <= 0.0
    {
        return None;
    }
    let scale_x = f64::from(target.0) / basis.0;
    let scale_y = f64::from(target.1) / basis.1;
    let scales = match fit {
        FitMode::Fill => (scale_x, scale_y),
        FitMode::Contain => {
            let scale = scale_x.min(scale_y);
            (scale, scale)
        }
        FitMode::Cover => {
            let scale = scale_x.max(scale_y);
            (scale, scale)
        }
    };
    Some((
        scaled_dimension_bound(surface.0, scales.0),
        scaled_dimension_bound(surface.1, scales.1),
    ))
}

pub(crate) fn shadow_intermediate_pixels(width: u32, height: u32, blur: f64) -> Option<u128> {
    if width == 0 || height == 0 || !blur.is_finite() || blur < 0.0 {
        return None;
    }
    let total_pad = u128::from(shadow_padding(blur)?).saturating_mul(2);
    Some(
        u128::from(width)
            .saturating_add(total_pad)
            .saturating_mul(u128::from(height).saturating_add(total_pad)),
    )
}

pub fn shadow_padding(blur: f64) -> Option<u64> {
    let padding = (blur * 2.0).ceil();
    if !padding.is_finite() || padding < 0.0 || padding > u64::MAX as f64 {
        return None;
    }
    Some(padding as u64)
}

pub fn shadow_extent_expansion(blur: f64, x: f64, y: f64) -> Option<(u64, u64)> {
    let padding = shadow_padding(blur)?.checked_mul(2)?;
    Some((
        padding.checked_add(offset_extent(x)?)?,
        padding.checked_add(offset_extent(y)?)?,
    ))
}

fn offset_extent(value: f64) -> Option<u64> {
    let extent = value.abs().ceil();
    if !extent.is_finite() || extent > u64::MAX as f64 {
        return None;
    }
    Some(extent as u64)
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

fn scaled_dimension_bound(extent: u32, scale: f64) -> u32 {
    (f64::from(extent) * scale).ceil().min(f64::from(u32::MAX)) as u32
}

fn shear_extent(extent: u128, shear: f64) -> Option<u128> {
    if !shear.is_finite() || shear.abs() > MAX_VISUAL_SHEAR {
        return None;
    }
    let fixed = (shear.abs() * SCALE_UNITS as f64).ceil() as u128;
    Some(ceil_div(extent.saturating_mul(fixed), SCALE_UNITS))
}

#[cfg(test)]
#[path = "visual_tests.rs"]
mod tests;

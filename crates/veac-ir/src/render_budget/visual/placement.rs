use crate::{Length, LengthUnit};

#[derive(Clone, Copy)]
pub struct CanvasPlacementFacts {
    pub generated: bool,
    pub effect_free: bool,
    pub frameless: bool,
    pub uncropped: bool,
    pub placement_default: bool,
    pub transform_anchor_centered: bool,
    pub position_zero: bool,
    pub scale_identity: bool,
    pub shear_zero: bool,
    pub rotation_zero: bool,
    pub shadowless: bool,
}

pub fn requires_canvas_placement(facts: CanvasPlacementFacts) -> bool {
    !facts.generated
        || !facts.effect_free
        || !facts.frameless
        || !facts.uncropped
        || !facts.placement_default
        || !facts.transform_anchor_centered
        || !facts.position_zero
        || !facts.scale_identity
        || !facts.shear_zero
        || !facts.rotation_zero
        || !facts.shadowless
}

pub fn length_pixels(value: Length, extent: u32) -> Option<u32> {
    let value = match value.unit {
        LengthUnit::Pixels => value.value,
        LengthUnit::Normalized => value.value * f64::from(extent),
        LengthUnit::Percent => value.value * f64::from(extent) / 100.0,
    };
    (value.is_finite() && value > 0.0 && value <= f64::from(u32::MAX))
        .then(|| value.round().max(1.0) as u32)
}

pub fn placement_intermediate_extent(
    canvas: (u32, u32),
    transformed: (u128, u128),
) -> Option<(u128, u128)> {
    if canvas.0 == 0 || canvas.1 == 0 || transformed.0 == 0 || transformed.1 == 0 {
        return None;
    }
    Some((
        transformed
            .0
            .saturating_mul(2)
            .saturating_add(u128::from(canvas.0)),
        transformed
            .1
            .saturating_mul(2)
            .saturating_add(u128::from(canvas.1)),
    ))
}

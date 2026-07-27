use veac_plan::canonical::{Anchor, Length, LengthUnit, Placement};
use veac_plan::EffectiveVisualProperties;

use super::{animation, time};

pub(crate) fn pixel_value(length: Length, extent: u32) -> f64 {
    match length.unit {
        LengthUnit::Pixels => length.value,
        LengthUnit::Normalized => length.value * f64::from(extent),
        LengthUnit::Percent => length.value * f64::from(extent) / 100.0,
    }
}

pub(crate) fn pixel_count(length: Length, extent: u32) -> u32 {
    let value = pixel_value(length, extent);
    value.round().clamp(1.0, f64::from(u32::MAX)) as u32
}

pub(crate) fn overlay_position(
    visual: &EffectiveVisualProperties,
    local_clock: &str,
    pivot_x: f64,
    pivot_y: f64,
) -> (String, String) {
    let (base_x, base_y) = placement_target(visual.placement);
    let offset_x = animation::point_x(&visual.transform.position, local_clock, "W");
    let offset_y = animation::point_y(&visual.transform.position, local_clock, "H");
    (
        format!("({base_x})+({offset_x})-{pivot_x}*w"),
        format!("({base_y})+({offset_y})-{pivot_y}*h"),
    )
}

fn placement_target(placement: Placement) -> (String, String) {
    match placement {
        Placement::Anchor { anchor, inset } => anchor_target(anchor, inset.x, inset.y),
        Placement::Absolute { position } => (
            length_expression(position.x, "W"),
            length_expression(position.y, "H"),
        ),
    }
}

fn anchor_target(anchor: Anchor, inset_x: f64, inset_y: f64) -> (String, String) {
    let x = time::number(inset_x);
    let y = time::number(inset_y);
    let left = x.clone();
    let center_x = format!("W/2+{x}");
    let right = format!("W-{x}");
    let top = y.clone();
    let center_y = format!("H/2+{y}");
    let bottom = format!("H-{y}");
    match anchor {
        Anchor::TopLeft => (left, top),
        Anchor::Top => (center_x, top),
        Anchor::TopRight => (right, top),
        Anchor::Left => (left, center_y),
        Anchor::Center => (center_x, center_y),
        Anchor::Right => (right, center_y),
        Anchor::BottomLeft => (left, bottom),
        Anchor::Bottom => (center_x, bottom),
        Anchor::BottomRight => (right, bottom),
    }
}

fn length_expression(length: Length, extent: &str) -> String {
    match length.unit {
        LengthUnit::Pixels => time::number(length.value),
        LengthUnit::Normalized => format!("{extent}*{}", time::number(length.value)),
        LengthUnit::Percent => format!("{extent}*{}/100", time::number(length.value)),
    }
}

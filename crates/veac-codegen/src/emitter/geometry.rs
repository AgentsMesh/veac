use veac_plan::canonical::{Anchor, Length, LengthUnit, Placement};
use veac_plan::EffectiveVisualProperties;
use veac_plan::ResolvedRenderPlan;

use super::{animation, process_owner::ProcessOwner, time, Canvas};

#[cfg(test)]
#[path = "../unit_tests/geometry_test_support.rs"]
mod test_support;
#[cfg(test)]
pub(crate) use test_support::overlay_position;

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

struct CoordinateSpace<'a> {
    width: &'a str,
    height: &'a str,
    source_width: &'a str,
    source_height: &'a str,
}

pub(super) fn canvas_position(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    visual: &EffectiveVisualProperties,
    local_clock: &str,
    pivot_x: f64,
    pivot_y: f64,
    canvas: Canvas,
) -> (String, String) {
    let width = canvas.width.to_string();
    let height = canvas.height.to_string();
    position(
        visual,
        plan,
        owner,
        local_clock,
        pivot_x,
        pivot_y,
        CoordinateSpace {
            width: &width,
            height: &height,
            source_width: "iw",
            source_height: "ih",
        },
    )
}

pub(super) fn canvas_overlay_position(
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    visual: &EffectiveVisualProperties,
    local_clock: &str,
    pivot_x: f64,
    pivot_y: f64,
    canvas: Canvas,
) -> (String, String) {
    let width = canvas.width.to_string();
    let height = canvas.height.to_string();
    position(
        visual,
        plan,
        owner,
        local_clock,
        pivot_x,
        pivot_y,
        CoordinateSpace {
            width: &width,
            height: &height,
            source_width: "w",
            source_height: "h",
        },
    )
}

fn position(
    visual: &EffectiveVisualProperties,
    plan: &ResolvedRenderPlan,
    owner: ProcessOwner<'_>,
    local_clock: &str,
    pivot_x: f64,
    pivot_y: f64,
    space: CoordinateSpace<'_>,
) -> (String, String) {
    let (base_x, base_y) = placement_target(visual.placement, space.width, space.height);
    let offset_x = animation::point_x(
        plan,
        owner,
        &visual.transform.position,
        local_clock,
        space.width,
    );
    let offset_y = animation::point_y(
        plan,
        owner,
        &visual.transform.position,
        local_clock,
        space.height,
    );
    (
        format!("({base_x})+({offset_x})-{pivot_x}*{}", space.source_width),
        format!("({base_y})+({offset_y})-{pivot_y}*{}", space.source_height),
    )
}

fn placement_target(placement: Placement, width: &str, height: &str) -> (String, String) {
    match placement {
        Placement::Anchor { anchor, inset } => {
            anchor_target(anchor, inset.x, inset.y, width, height)
        }
        Placement::Absolute { position } => (
            length_expression(position.x, width),
            length_expression(position.y, height),
        ),
    }
}

fn anchor_target(
    anchor: Anchor,
    inset_x: f64,
    inset_y: f64,
    width: &str,
    height: &str,
) -> (String, String) {
    let x = time::number(inset_x);
    let y = time::number(inset_y);
    let left = x.clone();
    let center_x = format!("{width}/2+{x}");
    let right = format!("{width}-{x}");
    let top = y.clone();
    let center_y = format!("{height}/2+{y}");
    let bottom = format!("{height}-{y}");
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

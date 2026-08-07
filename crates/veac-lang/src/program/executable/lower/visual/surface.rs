use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{Animatable, BlendMode, CardStyle, Compositing, Shadow};

use super::super::animation;
use super::{channel, malformed, ExecutableLowerError};
use crate::program::executable::lower::value;

pub(super) struct Surface {
    pub(super) opacity: Animatable<f64>,
    pub(super) compositing: Compositing,
    pub(super) card: Option<CardStyle>,
}

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
) -> Result<Surface, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::VisualSurface)?;
    if values.len() != 3 {
        return Err(malformed());
    }
    Ok(Surface {
        opacity: animation::percent(
            graph,
            &values[0],
            timebase,
            &channel(scope, "visual.opacity"),
        )?,
        compositing: compositing(graph, &values[1])?,
        card: card(graph, &values[2])?,
    })
}

fn compositing(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Compositing, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::Compositing)?;
    if values.len() != 2 {
        return Err(malformed());
    }
    Ok(Compositing {
        z_index: i32::try_from(value::integer(values.first())?).map_err(|_| malformed())?,
        blend_mode: blend(graph, &values[1])?,
    })
}

fn card(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<CardStyle>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::CardNone, []) => Ok(None),
        (Op::CardPresent, [radius, shadow]) => Ok(Some(CardStyle {
            corner_radius_pixels: animation::length_value(graph, Some(radius))?.value,
            shadow: shadow_value(graph, shadow)?,
        })),
        _ => Err(malformed()),
    }
}

fn shadow_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<Shadow>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ShadowNone, []) => Ok(None),
        (Op::ShadowPresent, [blur, opacity, offset, color]) => Ok(Some(Shadow {
            blur_pixels: animation::length_value(graph, Some(blur))?.value,
            opacity: animation::percent_value(graph, Some(opacity))?,
            offset: animation::vector_value(graph, Some(offset))?,
            color: value::color(Some(color))?,
        })),
        _ => Err(malformed()),
    }
}

pub(super) fn blend(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<BlendMode, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::BlendNormal => BlendMode::Normal,
        Op::BlendMultiply => BlendMode::Multiply,
        Op::BlendScreen => BlendMode::Screen,
        Op::BlendOverlay => BlendMode::Overlay,
        Op::BlendDarken => BlendMode::Darken,
        Op::BlendLighten => BlendMode::Lighten,
        Op::BlendColorDodge => BlendMode::ColorDodge,
        Op::BlendColorBurn => BlendMode::ColorBurn,
        Op::BlendHardLight => BlendMode::HardLight,
        Op::BlendSoftLight => BlendMode::SoftLight,
        Op::BlendDifference => BlendMode::Difference,
        Op::BlendExclusion => BlendMode::Exclusion,
        _ => return Err(malformed()),
    })
}

fn empty(graph: &FrozenDomainGraph, source: &Value) -> Result<Op, ExecutableLowerError> {
    let (operation, operands) = value::description(graph, source)?;
    operands
        .is_empty()
        .then_some(operation)
        .ok_or_else(malformed)
}

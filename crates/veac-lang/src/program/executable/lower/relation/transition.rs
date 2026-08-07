use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{
    CardinalDirection, CircleDirection, FadeColor, Transition, TransitionAlignment, TransitionKind,
    ZoomDirection,
};

use super::{malformed, ExecutableLowerError};
use crate::program::executable::lower::{animation, time, value};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
) -> Result<Transition, ExecutableLowerError> {
    let (operation, values) = value::description(graph, source)?;
    let duration = time::coordinate(values.first(), timebase)?;
    let kind = match (operation, values) {
        (Op::TransitionDissolve, [_]) => TransitionKind::Dissolve,
        (Op::TransitionFade, [_, color]) => TransitionKind::Fade {
            color: fade(graph, color)?,
        },
        (Op::TransitionWipe, [_, direction, angle, softness]) => TransitionKind::Wipe {
            direction: cardinal(graph, direction)?,
            angle_degrees: animation::angle_value(graph, Some(angle))?,
            softness: animation::percent_value(graph, Some(softness))?,
        },
        (Op::TransitionSlide, [_, direction, amount]) => TransitionKind::Slide {
            direction: cardinal(graph, direction)?,
            amount: value::finite(Some(amount))?,
        },
        (Op::TransitionZoom, [_, direction, amount]) => TransitionKind::Zoom {
            direction: zoom(graph, direction)?,
            amount: value::finite(Some(amount))?,
        },
        (Op::TransitionCircle, [_, direction, softness]) => TransitionKind::Circle {
            direction: circle(graph, direction)?,
            softness: animation::percent_value(graph, Some(softness))?,
        },
        (Op::TransitionPixelize, [_, amount]) => TransitionKind::Pixelize {
            amount: animation::percent_value(graph, Some(amount))?,
        },
        _ => return Err(malformed()),
    };
    Ok(Transition {
        kind,
        duration,
        alignment: TransitionAlignment::Centered,
    })
}

fn fade(graph: &FrozenDomainGraph, source: &Value) -> Result<FadeColor, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::FadeTransparent, []) => Ok(FadeColor::Transparent),
        (Op::FadeBlack, []) => Ok(FadeColor::Black),
        (Op::FadeWhite, []) => Ok(FadeColor::White),
        _ => Err(malformed()),
    }
}

fn cardinal(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<CardinalDirection, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::DirectionLeft, []) => Ok(CardinalDirection::Left),
        (Op::DirectionRight, []) => Ok(CardinalDirection::Right),
        (Op::DirectionUp, []) => Ok(CardinalDirection::Up),
        (Op::DirectionDown, []) => Ok(CardinalDirection::Down),
        _ => Err(malformed()),
    }
}

fn zoom(graph: &FrozenDomainGraph, source: &Value) -> Result<ZoomDirection, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::ZoomIn, []) => Ok(ZoomDirection::In),
        (Op::ZoomOut, []) => Ok(ZoomDirection::Out),
        _ => Err(malformed()),
    }
}

fn circle(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<CircleDirection, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::CircleOpen, []) => Ok(CircleDirection::Open),
        (Op::CircleClose, []) => Ok(CircleDirection::Close),
        _ => Err(malformed()),
    }
}

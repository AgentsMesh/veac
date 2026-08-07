use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::{ExactNumber, Value};
use crate::program::DomainOperationId as Op;
use veac_ir::{Length, LengthUnit, Point, Rect, Vec2};

use super::super::value;
use super::{invalid, ExecutableLowerError};

pub(in crate::program::executable::lower) fn scalar_value(
    _graph: &FrozenDomainGraph,
    source: Option<&Value>,
) -> Result<f64, ExecutableLowerError> {
    primitive(source, |value| match value {
        Value::Scalar(number) => Some(*number),
        _ => None,
    })
}

pub(in crate::program::executable::lower) fn length_value(
    _graph: &FrozenDomainGraph,
    source: Option<&Value>,
) -> Result<Length, ExecutableLowerError> {
    primitive(source, |value| match value {
        Value::Length(number) => Some(*number),
        _ => None,
    })
    .map(|value| Length {
        value,
        unit: LengthUnit::Pixels,
    })
}

pub(in crate::program::executable::lower) fn percent_value(
    _graph: &FrozenDomainGraph,
    source: Option<&Value>,
) -> Result<f64, ExecutableLowerError> {
    let percent = primitive(source, |value| match value {
        Value::Percent(number) => Some(*number),
        _ => None,
    })?;
    (0.0..=100.0)
        .contains(&percent)
        .then_some(percent / 100.0)
        .ok_or_else(invalid)
}

pub(in crate::program::executable::lower) fn angle_value(
    _graph: &FrozenDomainGraph,
    source: Option<&Value>,
) -> Result<f64, ExecutableLowerError> {
    primitive(source, |value| match value {
        Value::Angle(number) => Some(*number),
        _ => None,
    })
}

pub(in crate::program::executable::lower) fn point_value(
    graph: &FrozenDomainGraph,
    source: Option<&Value>,
) -> Result<Point, ExecutableLowerError> {
    let operands = description(graph, source, Op::Point, 2)?;
    Ok(Point {
        x: length_value(graph, operands.first())?,
        y: length_value(graph, operands.get(1))?,
    })
}

pub(in crate::program::executable::lower) fn vector_value(
    graph: &FrozenDomainGraph,
    source: Option<&Value>,
) -> Result<Vec2, ExecutableLowerError> {
    let operands = description(graph, source, Op::Vector, 2)?;
    Ok(Vec2 {
        x: scalar_value(graph, operands.first())?,
        y: scalar_value(graph, operands.get(1))?,
    })
}

pub(in crate::program::executable::lower) fn rect_value(
    graph: &FrozenDomainGraph,
    source: Option<&Value>,
) -> Result<Rect, ExecutableLowerError> {
    let operands = description(graph, source, Op::Rect, 4)?;
    Ok(Rect {
        x: scalar_value(graph, operands.first())?,
        y: scalar_value(graph, operands.get(1))?,
        width: scalar_value(graph, operands.get(2))?,
        height: scalar_value(graph, operands.get(3))?,
    })
}

fn primitive(
    source: Option<&Value>,
    select: impl FnOnce(&Value) -> Option<ExactNumber>,
) -> Result<f64, ExecutableLowerError> {
    let number = source.and_then(select).ok_or_else(invalid)?;
    value::safe_f64(number).ok_or_else(invalid)
}

fn description<'a>(
    graph: &'a FrozenDomainGraph,
    source: Option<&Value>,
    operation: Op,
    arity: usize,
) -> Result<&'a [Value], ExecutableLowerError> {
    let source = source.ok_or_else(invalid)?;
    let operands = value::description_operands(graph, source, operation)?;
    (operands.len() == arity)
        .then_some(operands)
        .ok_or_else(invalid)
}

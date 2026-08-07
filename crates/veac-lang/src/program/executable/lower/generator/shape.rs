use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{Paint, PathCommand, VectorGeometry, VectorShape, VectorStroke};

use super::super::error::ExecutableLowerError;
use super::super::value;
use super::{gradient, malformed, rect, vector};

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    geometry: &Value,
    fill: &Value,
    stroke: &Value,
) -> Result<VectorShape, ExecutableLowerError> {
    Ok(VectorShape {
        geometry: geometry_value(graph, geometry)?,
        fill: option_paint(graph, fill)?,
        stroke: stroke_value(graph, stroke)?,
    })
}

fn geometry_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<VectorGeometry, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::GeometryRectangle, [bounds]) => Ok(VectorGeometry::Rectangle {
            bounds: rect(graph, bounds)?,
        }),
        (Op::GeometryEllipse, [bounds]) => Ok(VectorGeometry::Ellipse {
            bounds: rect(graph, bounds)?,
        }),
        (Op::GeometryRoundedRectangle, [bounds, radius]) => Ok(VectorGeometry::RoundedRectangle {
            bounds: rect(graph, bounds)?,
            radius: value::finite(Some(radius))?,
        }),
        (Op::GeometryPolygon, [points]) => Ok(VectorGeometry::Polygon {
            points: value::list(Some(points))?
                .iter()
                .map(|point| vector(graph, point))
                .collect::<Result<_, _>>()?,
        }),
        (Op::GeometryPath, [commands]) => Ok(VectorGeometry::Path {
            commands: value::list(Some(commands))?
                .iter()
                .map(|command| path_command(graph, command))
                .collect::<Result<_, _>>()?,
        }),
        _ => Err(malformed()),
    }
}

fn path_command(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<PathCommand, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::PathMoveTo, [point]) => Ok(PathCommand::MoveTo {
            point: vector(graph, point)?,
        }),
        (Op::PathLineTo, [point]) => Ok(PathCommand::LineTo {
            point: vector(graph, point)?,
        }),
        (Op::PathClose, []) => Ok(PathCommand::Close),
        _ => Err(malformed()),
    }
}

fn paint(graph: &FrozenDomainGraph, source: &Value) -> Result<Paint, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::PaintSolid, [color]) => Ok(Paint::Solid {
            color: value::color(Some(color))?,
        }),
        (Op::PaintGradient, [gradient]) => Ok(Paint::Gradient {
            gradient: gradient::lower(graph, gradient)?,
        }),
        _ => Err(malformed()),
    }
}

fn option_paint(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<Paint>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::PaintNone, []) => Ok(None),
        (Op::PaintPresent, [paint_value]) => Ok(Some(paint(graph, paint_value)?)),
        _ => Err(malformed()),
    }
}

fn stroke_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<VectorStroke>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::StrokeNone, []) => Ok(None),
        (Op::StrokePresent, [paint_value, width]) => Ok(Some(VectorStroke {
            paint: paint(graph, paint_value)?,
            width_pixels: value::finite(Some(width))?,
        })),
        _ => Err(malformed()),
    }
}

use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{Animatable, Mask, MaskShape};

use super::super::animation;
use super::{channel, malformed, ExecutableLowerError};
use crate::program::executable::lower::value;

pub(super) fn lower(
    graph: &FrozenDomainGraph,
    source: &Value,
    timebase: u32,
    scope: &[&str],
    index: usize,
) -> Result<Mask, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::Mask)?;
    if values.len() != 4 {
        return Err(malformed());
    }
    let motion = value::description_operands(graph, &values[1], Op::MaskMotion)?;
    let edge = value::description_operands(graph, &values[2], Op::MaskEdge)?;
    if motion.len() != 3 || edge.len() != 2 {
        return Err(malformed());
    }
    let prefix = format!("visual.mask.{index}");
    Ok(Mask {
        shape: shape(graph, &values[0])?,
        position: animation::vector(
            graph,
            &motion[0],
            timebase,
            &channel(scope, &format!("{prefix}.position")),
        )?,
        scale: animation::vector(
            graph,
            &motion[1],
            timebase,
            &channel(scope, &format!("{prefix}.scale")),
        )?,
        rotation_degrees: animation::angle(
            graph,
            &motion[2],
            timebase,
            &channel(scope, &format!("{prefix}.rotation")),
        )?,
        feather_pixels: length_pixels(animation::length(
            graph,
            &edge[0],
            timebase,
            &channel(scope, &format!("{prefix}.feather")),
        )?),
        expansion_pixels: length_pixels(animation::length(
            graph,
            &edge[1],
            timebase,
            &channel(scope, &format!("{prefix}.expansion")),
        )?),
        invert: value::boolean(values.get(3))?,
    })
}

fn shape(graph: &FrozenDomainGraph, source: &Value) -> Result<MaskShape, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::MaskLinear, []) => Ok(MaskShape::Linear),
        (Op::MaskMirror, []) => Ok(MaskShape::Mirror),
        (Op::MaskCircle, []) => Ok(MaskShape::Circle),
        (Op::MaskRectangle, []) => Ok(MaskShape::Rectangle),
        (Op::MaskRoundedRectangle, [radius]) => Ok(MaskShape::RoundedRectangle {
            radius: animation::scalar_value(graph, Some(radius))?,
        }),
        (Op::MaskEllipse, []) => Ok(MaskShape::Ellipse),
        (Op::MaskPolygon, [points]) => Ok(MaskShape::Polygon {
            points: points_value(graph, points)?,
        }),
        (Op::MaskHeart, []) => Ok(MaskShape::Heart),
        (Op::MaskStar, []) => Ok(MaskShape::Star),
        (Op::MaskPath, [points]) => Ok(MaskShape::Path {
            points: points_value(graph, points)?,
        }),
        _ => Err(malformed()),
    }
}

fn points_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Vec<veac_ir::Vec2>, ExecutableLowerError> {
    value::list(Some(source))?
        .iter()
        .map(|point| animation::vector_value(graph, Some(point)))
        .collect()
}

fn length_pixels(source: Animatable<veac_ir::Length>) -> Animatable<f64> {
    match source {
        Animatable::Constant { value } => Animatable::constant(value.value),
        Animatable::Keyframes { keyframes } => Animatable::Keyframes {
            keyframes: keyframes
                .into_iter()
                .map(|keyframe| veac_ir::Keyframe {
                    id: keyframe.id,
                    time: keyframe.time,
                    value: keyframe.value.value,
                    interpolation: keyframe.interpolation,
                })
                .collect(),
        },
        Animatable::Binding { binding_id } => Animatable::Binding { binding_id },
    }
}

use crate::authoring::{MaskModifierDecl, MaskShapeDecl, NumberLiteral, VectorDecl};
use veac_ir::{Animatable, Mask, MaskShape, Vec2};

use super::context::Context;
use super::{animation, value};

pub(super) fn lower(ctx: &mut Context, declaration: &MaskModifierDecl) -> Option<Mask> {
    Some(Mask {
        shape: shape(ctx, &declaration.shape)?,
        position: match &declaration.position {
            Some(value) => animation::parameter(ctx, value, normalized_vector)?,
            None => Animatable::constant(Vec2 { x: 0.5, y: 0.5 }),
        },
        scale: match &declaration.scale {
            Some(value) => animation::parameter(ctx, value, scale_vector)?,
            None => Animatable::constant(Vec2 { x: 1.0, y: 1.0 }),
        },
        rotation_degrees: optional_scalar(ctx, declaration.rotation.as_ref(), "deg", 0.0)?,
        feather_pixels: optional_scalar(ctx, declaration.feather.as_ref(), "px", 0.0)?,
        expansion_pixels: optional_scalar(ctx, declaration.expansion.as_ref(), "px", 0.0)?,
        invert: declaration.invert.as_ref().is_some_and(|value| value.value),
    })
}

fn shape(ctx: &mut Context, value: &MaskShapeDecl) -> Option<MaskShape> {
    Some(match value {
        MaskShapeDecl::Linear => MaskShape::Linear,
        MaskShapeDecl::Mirror => MaskShape::Mirror,
        MaskShapeDecl::Circle => MaskShape::Circle,
        MaskShapeDecl::Rectangle => MaskShape::Rectangle,
        MaskShapeDecl::RoundedRectangle { radius, .. } => MaskShape::RoundedRectangle {
            radius: value::scalar(ctx, radius, "%")? / 100.0,
        },
        MaskShapeDecl::Ellipse => MaskShape::Ellipse,
        MaskShapeDecl::Polygon { points, .. } => MaskShape::Polygon {
            points: lower_points(ctx, points)?,
        },
        MaskShapeDecl::Heart => MaskShape::Heart,
        MaskShapeDecl::Star => MaskShape::Star,
        MaskShapeDecl::Path { points, .. } => MaskShape::Path {
            points: lower_points(ctx, points)?,
        },
    })
}

fn lower_points(ctx: &mut Context, points: &[VectorDecl]) -> Option<Vec<Vec2>> {
    points
        .iter()
        .map(|value| normalized_vector(ctx, value))
        .collect()
}

fn optional_scalar(
    ctx: &mut Context,
    declaration: Option<&crate::authoring::ParameterDecl<NumberLiteral>>,
    unit: &str,
    default: f64,
) -> Option<Animatable<f64>> {
    match declaration {
        Some(value) => animation::scalar(ctx, value, unit),
        None => Some(Animatable::constant(default)),
    }
}

fn normalized_vector(ctx: &mut Context, value: &VectorDecl) -> Option<Vec2> {
    Some(Vec2 {
        x: value::scale(ctx, &value.x)?,
        y: value::scale(ctx, &value.y)?,
    })
}

fn scale_vector(ctx: &mut Context, value: &VectorDecl) -> Option<Vec2> {
    normalized_vector(ctx, value)
}

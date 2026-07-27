use crate::authoring::{
    GeneratorDecl, GradientGeneratorDecl, GradientGeometryDecl, PaintDecl, PointDecl, StrokeDecl,
};
use veac_ir::{Generator, Gradient, GradientStop, Paint, Vec2, VectorShape, VectorStroke};

use super::context::Context;
use super::{color, generator_geometry, value};

pub(super) fn lower(ctx: &mut Context, declaration: &GeneratorDecl) -> Option<Generator> {
    Some(match declaration {
        GeneratorDecl::Transparent(_) => Generator::Transparent,
        GeneratorDecl::Silence(_) => Generator::Silence,
        GeneratorDecl::Solid { color: value, .. } => Generator::Solid {
            color: color::lower(ctx, value)?,
        },
        GeneratorDecl::Gradient(value) => Generator::Gradient {
            gradient: gradient(ctx, value)?,
        },
        GeneratorDecl::Shape(value) => Generator::Shape {
            shape: VectorShape {
                geometry: generator_geometry::lower(ctx, &value.geometry)?,
                fill: match &value.fill {
                    Some(paint) => Some(lower_paint(ctx, paint)?),
                    None => None,
                },
                stroke: match &value.stroke {
                    Some(stroke) => Some(lower_stroke(ctx, stroke)?),
                    None => None,
                },
            },
        },
    })
}

pub(super) fn gradient(ctx: &mut Context, value: &GradientGeneratorDecl) -> Option<Gradient> {
    let stops = value
        .stops
        .iter()
        .map(|stop| {
            Some(GradientStop {
                offset: percent(ctx, &stop.position, "gradient stop")?,
                color: color::lower(ctx, &stop.color)?,
            })
        })
        .collect::<Option<Vec<_>>>()?;
    Some(match &value.geometry {
        GradientGeometryDecl::Linear { from, to } => Gradient::Linear {
            start: point(ctx, from)?,
            end: point(ctx, to)?,
            stops,
        },
        GradientGeometryDecl::Radial { center, radius } => Gradient::Radial {
            center: point(ctx, center)?,
            radius: percent(ctx, radius, "radial gradient radius")?,
            stops,
        },
    })
}

fn lower_paint(ctx: &mut Context, value: &PaintDecl) -> Option<Paint> {
    Some(match value {
        PaintDecl::Solid(value) => Paint::Solid {
            color: color::lower(ctx, value)?,
        },
        PaintDecl::Gradient(value) => Paint::Gradient {
            gradient: gradient(ctx, value)?,
        },
    })
}

fn lower_stroke(ctx: &mut Context, value: &StrokeDecl) -> Option<VectorStroke> {
    Some(VectorStroke {
        paint: lower_paint(ctx, &value.paint)?,
        width_pixels: value::scalar(ctx, &value.width, "px")?,
    })
}

pub(super) fn percent(
    ctx: &mut Context,
    value: &crate::authoring::NumberLiteral,
    role: &str,
) -> Option<f64> {
    Some(value::number(ctx, value, &["%"], role)? / 100.0)
}

fn point(ctx: &mut Context, value: &PointDecl) -> Option<Vec2> {
    Some(Vec2 {
        x: percent(ctx, &value.x, "point x")?,
        y: percent(ctx, &value.y, "point y")?,
    })
}

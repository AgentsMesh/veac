use crate::authoring::{BoundsDecl, PathCommandDecl, PointDecl, ShapeGeometryDecl};
use veac_ir::{PathCommand, Rect, Vec2, VectorGeometry};

use super::context::Context;
use super::{generator, value};

pub(super) fn lower(ctx: &mut Context, value: &ShapeGeometryDecl) -> Option<VectorGeometry> {
    Some(match value {
        ShapeGeometryDecl::Rectangle(value) => VectorGeometry::Rectangle {
            bounds: bounds(ctx, value)?,
        },
        ShapeGeometryDecl::RoundedRectangle {
            bounds: value,
            radius,
        } => VectorGeometry::RoundedRectangle {
            bounds: bounds(ctx, value)?,
            radius: corner_radius(ctx, radius)?,
        },
        ShapeGeometryDecl::Ellipse(value) => VectorGeometry::Ellipse {
            bounds: bounds(ctx, value)?,
        },
        ShapeGeometryDecl::Polygon(points) => VectorGeometry::Polygon {
            points: points
                .iter()
                .map(|value| point(ctx, value))
                .collect::<Option<Vec<_>>>()?,
        },
        ShapeGeometryDecl::Path(commands) => VectorGeometry::Path {
            commands: commands
                .iter()
                .map(|command| lower_command(ctx, command))
                .collect::<Option<Vec<_>>>()?,
        },
    })
}

fn bounds(ctx: &mut Context, value: &BoundsDecl) -> Option<Rect> {
    Some(Rect {
        x: generator::percent(ctx, &value.x, "bounds x")?,
        y: generator::percent(ctx, &value.y, "bounds y")?,
        width: generator::percent(ctx, &value.width, "bounds width")?,
        height: generator::percent(ctx, &value.height, "bounds height")?,
    })
}

fn point(ctx: &mut Context, value: &PointDecl) -> Option<Vec2> {
    Some(Vec2 {
        x: generator::percent(ctx, &value.x, "point x")?,
        y: generator::percent(ctx, &value.y, "point y")?,
    })
}

fn lower_command(ctx: &mut Context, value: &PathCommandDecl) -> Option<PathCommand> {
    Some(match value {
        PathCommandDecl::Move(value) => PathCommand::MoveTo {
            point: point(ctx, value)?,
        },
        PathCommandDecl::Line(value) => PathCommand::LineTo {
            point: point(ctx, value)?,
        },
        PathCommandDecl::Close(_) => PathCommand::Close,
    })
}

fn corner_radius(ctx: &mut Context, value: &crate::authoring::NumberLiteral) -> Option<f64> {
    if value.raw.ends_with('%') {
        generator::percent(ctx, value, "rounded rectangle radius")
    } else {
        Some(value::scalar(ctx, value, "px")? / ctx.canvas_width.min(ctx.canvas_height))
    }
}

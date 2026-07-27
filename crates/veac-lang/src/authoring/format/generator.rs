use crate::authoring::{
    BoundsDecl, GeneratorDecl, GradientGeneratorDecl, GradientGeometryDecl, PaintDecl,
    PathCommandDecl, ShapeGeneratorDecl, ShapeGeometryDecl,
};

use super::writer::Writer;

pub(super) fn source(writer: &mut Writer, value: &GeneratorDecl) {
    match value {
        GeneratorDecl::Transparent(_) => writer.line("source generated transparent;"),
        GeneratorDecl::Silence(_) => writer.line("source generated silence;"),
        GeneratorDecl::Solid { color, .. } => writer.block("source generated solid", |writer| {
            writer.line(format!("color {};", color.value));
        }),
        GeneratorDecl::Gradient(value) => gradient(writer, "source generated gradient", value),
        GeneratorDecl::Shape(value) => shape(writer, value),
    }
}

fn gradient(writer: &mut Writer, prefix: &str, value: &GradientGeneratorDecl) {
    let kind = match &value.geometry {
        GradientGeometryDecl::Linear { .. } => "linear",
        GradientGeometryDecl::Radial { .. } => "radial",
    };
    writer.block(format!("{prefix} {kind}"), |writer| {
        match &value.geometry {
            GradientGeometryDecl::Linear { from, to } => {
                point_line(writer, "from", from);
                point_line(writer, "to", to);
            }
            GradientGeometryDecl::Radial { center, radius } => {
                point_line(writer, "center", center);
                writer.line(format!("radius {};", radius.raw));
            }
        }
        for stop in &value.stops {
            writer.line(format!("stop {} {};", stop.position.raw, stop.color.value));
        }
    });
}

fn shape(writer: &mut Writer, value: &ShapeGeneratorDecl) {
    writer.block("source generated shape", |writer| {
        geometry(writer, &value.geometry);
        if let Some(value) = &value.fill {
            paint(writer, "fill", value);
        }
        if let Some(value) = &value.stroke {
            paint(writer, &format!("stroke {}", value.width.raw), &value.paint);
        }
    });
}

fn geometry(writer: &mut Writer, value: &ShapeGeometryDecl) {
    match value {
        ShapeGeometryDecl::Rectangle(bounds) => {
            bounds_block(writer, "geometry rectangle", bounds, None)
        }
        ShapeGeometryDecl::RoundedRectangle { bounds, radius } => bounds_block(
            writer,
            "geometry rounded-rectangle",
            bounds,
            Some(&radius.raw),
        ),
        ShapeGeometryDecl::Ellipse(bounds) => {
            bounds_block(writer, "geometry ellipse", bounds, None)
        }
        ShapeGeometryDecl::Polygon(points) => writer.block("geometry polygon", |writer| {
            for point in points {
                point_line(writer, "point", point);
            }
        }),
        ShapeGeometryDecl::Path(commands) => writer.block("geometry path", |writer| {
            for command in commands {
                match command {
                    PathCommandDecl::Move(point) => point_line(writer, "move", point),
                    PathCommandDecl::Line(point) => point_line(writer, "line", point),
                    PathCommandDecl::Close(_) => writer.line("close;"),
                }
            }
        }),
    }
}

fn bounds_block(writer: &mut Writer, header: &str, value: &BoundsDecl, radius: Option<&str>) {
    writer.block(header, |writer| {
        writer.line(format!(
            "bounds {} {} {} {};",
            value.x.raw, value.y.raw, value.width.raw, value.height.raw
        ));
        if let Some(radius) = radius {
            writer.line(format!("radius {radius};"));
        }
    });
}

fn paint(writer: &mut Writer, prefix: &str, value: &PaintDecl) {
    match value {
        PaintDecl::Solid(color) => writer.line(format!("{prefix} solid {};", color.value)),
        PaintDecl::Gradient(value) => gradient(writer, &format!("{prefix} gradient"), value),
    }
}

fn point_line(writer: &mut Writer, name: &str, point: &crate::authoring::PointDecl) {
    writer.line(format!("{name} {} {};", point.x.raw, point.y.raw));
}

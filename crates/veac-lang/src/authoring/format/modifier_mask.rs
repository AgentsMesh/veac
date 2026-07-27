use crate::authoring::{MaskModifierDecl, MaskShapeDecl};

use super::parameter::{scalar, vector};
use super::writer::Writer;

pub(super) fn mask(writer: &mut Writer, value: &MaskModifierDecl) {
    writer.block(format!("mask {}", value.id.value), |writer| {
        shape(writer, &value.shape);
        if let Some(value) = &value.position {
            vector(writer, "position", value);
        }
        if let Some(value) = &value.scale {
            vector(writer, "scale", value);
        }
        if let Some(value) = &value.rotation {
            scalar(writer, "rotation", value);
        }
        if let Some(value) = &value.feather {
            scalar(writer, "feather", value);
        }
        if let Some(value) = &value.expansion {
            scalar(writer, "expansion", value);
        }
        if let Some(value) = &value.invert {
            writer.line(format!("invert {};", value.value));
        }
    });
}

fn shape(writer: &mut Writer, value: &MaskShapeDecl) {
    let name = match value {
        MaskShapeDecl::Linear => "linear",
        MaskShapeDecl::Mirror => "mirror",
        MaskShapeDecl::Circle => "circle",
        MaskShapeDecl::Rectangle => "rectangle",
        MaskShapeDecl::RoundedRectangle { radius, .. } => {
            writer.block("shape rounded-rectangle", |writer| {
                writer.line(format!("radius {};", radius.raw));
            });
            return;
        }
        MaskShapeDecl::Ellipse => "ellipse",
        MaskShapeDecl::Polygon { points, .. } => {
            point_shape(writer, "polygon", points);
            return;
        }
        MaskShapeDecl::Heart => "heart",
        MaskShapeDecl::Star => "star",
        MaskShapeDecl::Path { points, .. } => {
            point_shape(writer, "path", points);
            return;
        }
    };
    writer.line(format!("shape {name};"));
}

fn point_shape(writer: &mut Writer, name: &str, points: &[crate::authoring::VectorDecl]) {
    writer.block(format!("shape {name}"), |writer| {
        for point in points {
            writer.block("point", |writer| {
                writer.line(format!("x {};", point.x.raw));
                writer.line(format!("y {};", point.y.raw));
            });
        }
    });
}

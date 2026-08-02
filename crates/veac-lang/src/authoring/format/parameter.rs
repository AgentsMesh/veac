use crate::authoring::{
    NumberLiteral, ParameterDecl, ParameterKey, PointDecl, RectDecl, VectorDecl,
};

use super::writer::Writer;

pub(super) fn scalar(writer: &mut Writer, name: &str, value: &ParameterDecl<NumberLiteral>) {
    match value {
        ParameterDecl::Constant(value) => writer.line(format!("{name} {};", value.raw)),
        ParameterDecl::Curve { keys, .. } => curve(writer, name, keys, |writer, value| {
            writer.line(format!("value {};", value.raw));
        }),
    }
}

pub(super) fn point(writer: &mut Writer, name: &str, value: &ParameterDecl<PointDecl>) {
    match value {
        ParameterDecl::Constant(value) => point_value(writer, name, value),
        ParameterDecl::Curve { keys, .. } => curve(writer, name, keys, |writer, value| {
            point_value(writer, "value", value);
        }),
    }
}

pub(super) fn vector(writer: &mut Writer, name: &str, value: &ParameterDecl<VectorDecl>) {
    match value {
        ParameterDecl::Constant(value) => vector_value(writer, name, value),
        ParameterDecl::Curve { keys, .. } => curve(writer, name, keys, |writer, value| {
            vector_value(writer, "value", value);
        }),
    }
}

pub(super) fn rect(writer: &mut Writer, name: &str, value: &ParameterDecl<RectDecl>) {
    match value {
        ParameterDecl::Constant(value) => rect_value(writer, name, value),
        ParameterDecl::Curve { keys, .. } => curve(writer, name, keys, |writer, value| {
            rect_value(writer, "value", value);
        }),
    }
}

fn curve<T>(
    writer: &mut Writer,
    name: &str,
    keys: &[ParameterKey<T>],
    format_value: impl Fn(&mut Writer, &T),
) {
    writer.block(format!("{name} curve"), |writer| {
        for key in keys {
            writer.block(format!("key {}", key.id.value), |writer| {
                writer.line(format!("at {};", key.at.raw));
                format_value(writer, &key.value);
                super::interpolation::write(writer, &key.interpolation);
            });
        }
    });
}

fn point_value(writer: &mut Writer, name: &str, value: &PointDecl) {
    writer.block(name, |writer| {
        writer.line(format!("x {};", value.x.raw));
        writer.line(format!("y {};", value.y.raw));
    });
}

fn vector_value(writer: &mut Writer, name: &str, value: &VectorDecl) {
    writer.block(name, |writer| {
        writer.line(format!("x {};", value.x.raw));
        writer.line(format!("y {};", value.y.raw));
    });
}

fn rect_value(writer: &mut Writer, name: &str, value: &RectDecl) {
    writer.block(name, |writer| {
        writer.line(format!("x {};", value.x.raw));
        writer.line(format!("y {};", value.y.raw));
        writer.line(format!("width {};", value.width.raw));
        writer.line(format!("height {};", value.height.raw));
    });
}

use crate::authoring::{ColorStageDecl, ColorWheelDecl};

use super::writer::Writer;

pub(super) fn stage(writer: &mut Writer, value: &ColorStageDecl) {
    match value {
        ColorStageDecl::Basic(value) => writer.block("basic", |writer| {
            number(writer, "exposure", &value.exposure);
            number(writer, "highlights", &value.highlights);
            number(writer, "shadows", &value.shadows);
            number(writer, "temperature", &value.temperature);
            number(writer, "tint", &value.tint);
            number(writer, "fade", &value.fade);
        }),
        ColorStageDecl::Matrix(value) => writer.block("matrix", |writer| {
            triple(writer, "red", &value.rows[0]);
            triple(writer, "green", &value.rows[1]);
            triple(writer, "blue", &value.rows[2]);
            triple(writer, "offset", &value.offset);
        }),
        ColorStageDecl::Hsl(value) => writer.block("hsl", |writer| {
            writer.line(format!("range {};", value.range.value));
            number(writer, "hue", &value.hue);
            number(writer, "saturation", &value.saturation);
            number(writer, "lightness", &value.lightness);
        }),
        ColorStageDecl::Curves(value) => writer.block("curves", |writer| {
            writer.line(format!("interpolation {};", value.interpolation.value));
            curve(writer, "luma", value.luma.as_deref());
            curve(writer, "red", value.red.as_deref());
            curve(writer, "green", value.green.as_deref());
            curve(writer, "blue", value.blue.as_deref());
        }),
        ColorStageDecl::Wheels(value) => writer.block("wheels", |writer| {
            wheel(writer, "lift", &value.lift);
            wheel(writer, "gamma", &value.gamma);
            wheel(writer, "gain", &value.gain);
        }),
        ColorStageDecl::Lut(value) => writer.block("lut", |writer| {
            writer.line(format!("resource {};", value.resource.value));
            writer.line(format!("interpolation {};", value.interpolation.value));
        }),
    }
}

fn curve(
    writer: &mut Writer,
    channel: &str,
    points: Option<&[crate::authoring::ColorCurvePointDecl]>,
) {
    if let Some(points) = points {
        writer.block(format!("curve {channel}"), |writer| {
            for point in points {
                writer.line(format!("point {} {};", point.input.raw, point.output.raw));
            }
        });
    }
}

fn number(writer: &mut Writer, name: &str, value: &crate::authoring::NumberLiteral) {
    writer.line(format!("{name} {};", value.raw));
}

fn triple(writer: &mut Writer, name: &str, value: &[crate::authoring::NumberLiteral; 3]) {
    writer.line(format!(
        "{name} {} {} {};",
        value[0].raw, value[1].raw, value[2].raw
    ));
}

fn wheel(writer: &mut Writer, name: &str, value: &ColorWheelDecl) {
    writer.line(format!(
        "{name} {} {} {};",
        value.red.raw, value.green.raw, value.blue.raw
    ));
}

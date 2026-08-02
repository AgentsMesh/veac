use crate::authoring::TextLayoutDecl;
use veac_ir::{
    HorizontalTextAlignment, TextOrientation, TextOverflow, TextPathAlignment, TextWrap,
    TextWritingMode, VerticalTextAlignment,
};

use super::text::number;
use super::writer::Writer;

pub(super) fn layout(writer: &mut Writer, value: &TextLayoutDecl) {
    writer.block("layout", |writer| {
        number(writer, "box-width", value.box_width.as_ref());
        number(writer, "box-height", value.box_height.as_ref());
        line(
            writer,
            "wrap",
            value.wrap.as_ref().map(|value| match value.value {
                TextWrap::None => "none",
                TextWrap::Word => "word",
                TextWrap::Character => "character",
            }),
        );
        line(
            writer,
            "overflow",
            value.overflow.as_ref().map(|value| match value.value {
                TextOverflow::Visible => "visible",
                TextOverflow::Clip => "clip",
                TextOverflow::Ellipsis => "ellipsis",
            }),
        );
        line(
            writer,
            "horizontal-align",
            value
                .horizontal_alignment
                .as_ref()
                .map(|value| match value.value {
                    HorizontalTextAlignment::Left => "left",
                    HorizontalTextAlignment::Center => "center",
                    HorizontalTextAlignment::Right => "right",
                }),
        );
        line(
            writer,
            "vertical-align",
            value
                .vertical_alignment
                .as_ref()
                .map(|value| match value.value {
                    VerticalTextAlignment::Top => "top",
                    VerticalTextAlignment::Middle => "middle",
                    VerticalTextAlignment::Bottom => "bottom",
                }),
        );
        line(
            writer,
            "writing-mode",
            value.writing_mode.as_ref().map(|value| match value.value {
                TextWritingMode::HorizontalTb => "horizontal-tb",
                TextWritingMode::VerticalRl => "vertical-rl",
                TextWritingMode::VerticalLr => "vertical-lr",
            }),
        );
        line(
            writer,
            "orientation",
            value.orientation.as_ref().map(|value| match value.value {
                TextOrientation::Upright => "upright",
                TextOrientation::Sideways => "sideways",
                TextOrientation::Mixed => "mixed",
            }),
        );
        if let Some(value) = &value.path {
            path(writer, value);
        }
    });
}

fn path(writer: &mut Writer, value: &crate::authoring::TextPathDecl) {
    writer.block("path", |writer| {
        for point in &value.points {
            writer.block("point", |writer| {
                writer.line(format!("x {};", point.x.raw));
                writer.line(format!("y {};", point.y.raw));
            });
        }
        writer.line(format!("start-offset {};", value.start_offset.raw));
        writer.line(format!("reverse {};", value.reverse.value));
        writer.line(format!(
            "align {};",
            match value.align.value {
                TextPathAlignment::Start => "start",
                TextPathAlignment::Center => "center",
                TextPathAlignment::End => "end",
            }
        ));
    });
}

fn line(writer: &mut Writer, name: &str, value: Option<&str>) {
    if let Some(value) = value {
        writer.line(format!("{name} {value};"));
    }
}

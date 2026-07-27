use crate::authoring::{TextAnimationDecl, TextGranularityDecl};

use super::parameter::{point, scalar, vector};
use super::writer::Writer;

pub(super) fn animation(writer: &mut Writer, value: &TextAnimationDecl) {
    writer.block("animation", |writer| {
        writer.line(format!(
            "unit {};",
            match value.unit.value {
                TextGranularityDecl::Whole => "whole",
                TextGranularityDecl::Line => "line",
                TextGranularityDecl::Word => "word",
                TextGranularityDecl::Grapheme => "grapheme",
            }
        ));
        writer.line(format!("stagger {};", value.stagger.raw));
        if let Some(value) = &value.reveal {
            scalar(writer, "reveal", value);
        }
        if let Some(value) = &value.highlight {
            writer.block("highlight", |writer| {
                writer.line(format!("fill {};", value.fill.value));
                scalar(writer, "progress", &value.progress);
            });
        }
        if let Some(value) = &value.opacity {
            scalar(writer, "opacity", value);
        }
        if let Some(value) = &value.transform {
            writer.block("transform", |writer| {
                if let Some(value) = &value.position {
                    point(writer, "position", value);
                }
                if let Some(value) = &value.scale {
                    vector(writer, "scale", value);
                }
                if let Some(value) = &value.rotation {
                    scalar(writer, "rotation", value);
                }
            });
        }
    });
}

use crate::authoring::{InterpolationDecl, InterpolationKind};

use super::writer::Writer;

pub fn write(writer: &mut Writer, value: &InterpolationDecl) {
    let token = match &value.kind {
        InterpolationKind::Hold => "hold",
        InterpolationKind::Linear => "linear",
        InterpolationKind::EaseIn => "ease-in",
        InterpolationKind::EaseOut => "ease-out",
        InterpolationKind::EaseInOut => "ease-in-out",
        InterpolationKind::CubicBezier { x1, y1, x2, y2 } => {
            writer.block("interpolation cubic-bezier", |writer| {
                writer.line(format!("x1 {};", x1.raw));
                writer.line(format!("y1 {};", y1.raw));
                writer.line(format!("x2 {};", x2.raw));
                writer.line(format!("y2 {};", y2.raw));
            });
            return;
        }
    };
    writer.line(format!("interpolation {token};"));
}

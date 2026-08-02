use crate::authoring::ShadowDecl;

use super::writer::Writer;

pub(super) fn shadow(writer: &mut Writer, value: &ShadowDecl) {
    writer.block("shadow", |writer| {
        writer.line(format!("color {};", value.color.value));
        writer.line(format!("opacity {};", value.opacity.raw));
        writer.line(format!("blur {};", value.blur.raw));
        writer.block("offset", |writer| {
            writer.line(format!("x {};", value.offset.x.raw));
            writer.line(format!("y {};", value.offset.y.raw));
        });
    });
}

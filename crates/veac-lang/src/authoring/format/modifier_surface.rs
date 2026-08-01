use crate::authoring::SurfaceModifierDecl;

use super::writer::Writer;

pub(super) fn surface(writer: &mut Writer, value: &SurfaceModifierDecl) {
    writer.block(format!("surface {}", value.id.value), |writer| {
        writer.line(format!("corner-radius {};", value.corner_radius.raw));
        if let Some(value) = &value.shadow {
            super::shadow::shadow(writer, value);
        }
    });
}

use crate::authoring::{ResourceDecl, ResourceKind, ResourceLocator, StreamSelection};

use super::value::quoted;
use super::writer::Writer;

pub(super) fn resource(writer: &mut Writer, value: &ResourceDecl) {
    writer.block(
        format!("resource {} {}", kind(value.kind), value.id.value),
        |writer| {
            match &value.locator {
                ResourceLocator::Local { path, .. } => writer.block("locator local", |writer| {
                    writer.line(format!("path {};", quoted(&path.value)));
                }),
                ResourceLocator::Remote { uri, identity, .. } => {
                    writer.block("locator remote", |writer| {
                        writer.line(format!("uri {};", quoted(&uri.value)));
                        let crate::authoring::ResourceIdentity::Sha256(value) = identity;
                        writer.line(format!("identity sha256 {};", quoted(&value.value)));
                    })
                }
            }
            if let Some(streams) = &value.streams {
                writer.block("streams", |writer| {
                    writer.line(format!("video {};", stream(streams.video)));
                    writer.line(format!("audio {};", stream(streams.audio)));
                });
            }
        },
    );
}

fn kind(value: ResourceKind) -> &'static str {
    match value {
        ResourceKind::Video => "video",
        ResourceKind::Audio => "audio",
        ResourceKind::Image => "image",
        ResourceKind::Font => "font",
        ResourceKind::Lut1d => "lut-1d",
        ResourceKind::Lut3d => "lut-3d",
    }
}

fn stream(value: StreamSelection) -> String {
    match value {
        StreamSelection::Auto => "auto".to_owned(),
        StreamSelection::Disabled => "disabled".to_owned(),
    }
}

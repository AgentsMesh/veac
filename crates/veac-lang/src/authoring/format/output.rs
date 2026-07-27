use crate::authoring::{OutputDecl, OutputEncoding};

use super::output_aux::{audio_stem, caption_sidecar, image_sequence, scope};
use super::output_video::video;
use super::value::quoted;
use super::writer::Writer;

pub(super) fn output(writer: &mut Writer, value: &OutputDecl) {
    writer.block(
        format!("output {} {}", value.encoding.kind(), value.id.value),
        |writer| {
            writer.line(format!("sequence {};", value.sequence.value));
            writer.line(format!("file-name {};", quoted(&value.file_name.value)));
            writer.block("encoding", |writer| encoding(writer, &value.encoding));
        },
    );
}

fn encoding(writer: &mut Writer, value: &OutputEncoding) {
    match value {
        OutputEncoding::Video(value) => video(writer, value),
        OutputEncoding::ImageSequence(value) => image_sequence(writer, value),
        OutputEncoding::CaptionSidecar(value) => caption_sidecar(writer, value),
        OutputEncoding::AudioStem(value) => audio_stem(writer, value),
        OutputEncoding::Scope(value) => scope(writer, value),
    }
}

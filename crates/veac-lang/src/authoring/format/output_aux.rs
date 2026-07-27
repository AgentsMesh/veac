use crate::authoring::{
    AudioStemEncoding, AudioStemSourceDecl, CaptionSidecarEncoding, ImageSequenceEncoding,
    OutputKeyword, ScopeEncoding,
};

use super::writer::Writer;

pub(super) fn image_sequence(writer: &mut Writer, value: &ImageSequenceEncoding) {
    writer.line(format!("format {};", value.format.token()));
    writer.line(format!("start-number {};", value.start_number));
}

pub(super) fn caption_sidecar(writer: &mut Writer, value: &CaptionSidecarEncoding) {
    writer.line(format!("format {};", value.format.token()));
    writer.block("tracks", |writer| {
        for id in &value.track_ids {
            writer.line(format!("track {};", id.value));
        }
    });
}

pub(super) fn audio_stem(writer: &mut Writer, value: &AudioStemEncoding) {
    writer.line(format!("format {};", value.format.token()));
    writer.block("audio", |writer| {
        writer.line(format!("codec {};", value.audio.codec.token()));
        writer.line(format!("sample-rate {};", value.audio.sample_rate));
        writer.line(format!("channels {};", value.audio.channels));
    });
    writer.line(format!("source {};", stem_source(&value.source)));
}

pub(super) fn scope(writer: &mut Writer, value: &ScopeEncoding) {
    writer.line(format!("scope {};", value.scope.token()));
    writer.line(format!("at {};", value.at.raw));
    writer.line(format!("width {};", value.width));
    writer.line(format!("height {};", value.height));
    writer.line(format!("format {};", value.format.token()));
}

fn stem_source(value: &AudioStemSourceDecl) -> String {
    match value {
        AudioStemSourceDecl::Master => "master".into(),
        AudioStemSourceDecl::Track(id) => format!("track {}", id.value),
        AudioStemSourceDecl::Bus(id) => format!("bus {}", id.value),
    }
}

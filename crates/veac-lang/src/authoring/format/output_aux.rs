use crate::authoring::{
    AudioMixSourceDecl, AudioStemEncoding, AudioStemFormat, CaptionSidecarEncoding,
    ImageSequenceEncoding, OutputKeyword, ScopeEncoding,
};

use super::output_units::{channel_layout, sample_rate};
use super::writer::Writer;

pub(super) fn image_sequence(writer: &mut Writer, value: &ImageSequenceEncoding) {
    writer.line(format!("numbering from {};", value.start_number));
    writer.line(format!("encode {};", value.format.token()));
}

pub(super) fn caption_sidecar(writer: &mut Writer, value: &CaptionSidecarEncoding) {
    writer.block("source caption-tracks", |writer| {
        for id in &value.track_ids {
            writer.line(format!("track {};", id.value));
        }
    });
    writer.line(format!("encode {};", value.format.token()));
}

pub(super) fn audio_stem(writer: &mut Writer, value: &AudioStemEncoding) {
    writer.line(format!("source {};", mix_source(&value.source)));
    writer.block(format!("encode {}", value.format.token()), |writer| {
        if value.format == AudioStemFormat::Wav {
            writer.line(format!("sample-format {};", value.audio.codec.token()));
        }
        writer.line(format!(
            "sample-rate {};",
            sample_rate(value.audio.sample_rate)
        ));
        writer.line(format!(
            "channel-layout {};",
            channel_layout(value.audio.channels)
        ));
    });
}

pub(super) fn scope(writer: &mut Writer, value: &ScopeEncoding) {
    writer.line(format!("analyze {};", value.scope.token()));
    writer.line(format!("frame containing {};", value.at.raw));
    writer.line(format!("canvas {}px by {}px;", value.width, value.height));
    writer.line(format!("encode {};", value.format.token()));
}

pub(super) fn mix_source(value: &AudioMixSourceDecl) -> String {
    match value {
        AudioMixSourceDecl::Master => "master".into(),
        AudioMixSourceDecl::Track(id) => format!("track {}", id.value),
        AudioMixSourceDecl::Bus(id) => format!("bus {}", id.value),
    }
}

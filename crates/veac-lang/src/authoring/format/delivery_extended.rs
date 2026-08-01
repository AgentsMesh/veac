mod hls;

use crate::authoring::{
    AnimatedImageRecipe, AudioChannelLayoutDecl, AudioFileRecipe, GifPlaybackDecl, OutputKeyword,
    StillImageRecipe,
};

use super::output_aux::mix_source;
use super::writer::Writer;

pub(super) fn audio_file(writer: &mut Writer, value: &AudioFileRecipe) {
    writer.line(format!("source {};", mix_source(&value.source)));
    writer.block("encode mp3", |writer| {
        writer.line(format!("bitrate {};", value.encoding.bitrate.raw));
        writer.line(format!("sample-rate {};", value.encoding.sample_rate.raw));
        writer.line(format!(
            "channel-layout {};",
            channel_layout(value.encoding.channel_layout)
        ));
    });
}

pub(super) fn animated_image(writer: &mut Writer, value: &AnimatedImageRecipe) {
    writer.block("encode gif", |writer| {
        writer.line(format!("playback {};", playback(value.playback)));
        writer.line(format!("dither {};", value.dither.token()));
    });
}

pub(super) fn still_image(writer: &mut Writer, value: &StillImageRecipe) {
    writer.line(format!("frame containing {};", value.at.raw));
    writer.line(format!("encode {};", value.format.token()));
}

pub(super) fn adaptive_package(
    writer: &mut Writer,
    value: &crate::authoring::AdaptivePackageRecipe,
) {
    hls::package(writer, value);
}

pub(super) fn channel_layout(value: AudioChannelLayoutDecl) -> &'static str {
    match value {
        AudioChannelLayoutDecl::Mono => "mono",
        AudioChannelLayoutDecl::Stereo => "stereo",
    }
}

fn playback(value: GifPlaybackDecl) -> String {
    match value {
        GifPlaybackDecl::Once => "once".into(),
        GifPlaybackDecl::Forever => "forever".into(),
        GifPlaybackDecl::Times(count) => format!("{count}times"),
    }
}

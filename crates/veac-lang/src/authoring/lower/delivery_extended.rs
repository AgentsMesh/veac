mod hls;

use crate::authoring::{
    AnimatedImageRecipe, AudioChannelLayoutDecl, AudioFileRecipe, GifDither, GifPlaybackDecl,
    StillImageRecipe,
};
use veac_ir::{
    AnimatedImage, AudioChannelLayout, AudioFile, AudioFileEncoding, DeliverableKind,
    FrameSelection, GifAnimation, GifPlayback, Mp3Encoding, StillImage,
};

use super::{context::Context, delivery::stem_source, delivery_units, output_types};

pub(super) fn audio_file(ctx: &mut Context, value: &AudioFileRecipe) -> Option<DeliverableKind> {
    Some(DeliverableKind::AudioFile(AudioFile {
        source: stem_source(ctx, &value.source)?,
        encoding: AudioFileEncoding::Mp3(Mp3Encoding {
            bitrate_bps: delivery_units::bitrate_u32(ctx, &value.encoding.bitrate, "MP3 bitrate")?,
            sample_rate_hz: delivery_units::sample_rate(
                ctx,
                &value.encoding.sample_rate,
                "MP3 sample-rate",
            )?,
            channel_layout: channel_layout(value.encoding.channel_layout),
        }),
    }))
}

pub(super) fn animated_image(value: &AnimatedImageRecipe) -> DeliverableKind {
    DeliverableKind::AnimatedImage(AnimatedImage::Gif(GifAnimation {
        playback: match value.playback {
            GifPlaybackDecl::Once => GifPlayback::Once,
            GifPlaybackDecl::Forever => GifPlayback::Forever,
            GifPlaybackDecl::Times(count) => GifPlayback::Times { count },
        },
        dither: match value.dither {
            GifDither::Bayer => veac_ir::GifDither::Bayer,
            GifDither::FloydSteinberg => veac_ir::GifDither::FloydSteinberg,
            GifDither::Sierra2 => veac_ir::GifDither::Sierra2,
            GifDither::None => veac_ir::GifDither::None,
        },
    }))
}

pub(super) fn still_image(ctx: &mut Context, value: &StillImageRecipe) -> Option<DeliverableKind> {
    Some(DeliverableKind::StillImage(StillImage {
        frame: FrameSelection::Containing {
            at: super::value::time(ctx, &value.at)?,
        },
        encoding: output_types::image_format(value.format),
    }))
}

pub(super) fn adaptive_package(
    ctx: &mut Context,
    value: &crate::authoring::AdaptivePackageRecipe,
) -> Option<DeliverableKind> {
    hls::lower(ctx, value)
}

pub(super) fn channel_layout(value: AudioChannelLayoutDecl) -> AudioChannelLayout {
    match value {
        AudioChannelLayoutDecl::Mono => AudioChannelLayout::Mono,
        AudioChannelLayoutDecl::Stereo => AudioChannelLayout::Stereo,
    }
}

use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use veac_ir::{
    AnimatedImage, AudioFile, AudioFileEncoding, AudioStemOutput, CaptionSidecarOutput,
    DeliverableKind, FrameSelection, GifAnimation, ImageSequenceOutput, Mp3Encoding, ScopeOutput,
    StillImage,
};

use super::super::{time, value};
use super::{common, malformed, ExecutableLowerError};

mod formats;

pub(super) fn image_frames(
    graph: &FrozenDomainGraph,
    values: &[Value],
) -> Result<DeliverableKind, ExecutableLowerError> {
    let [_, _, format, start] = values else {
        return Err(malformed());
    };
    Ok(DeliverableKind::ImageSequence(ImageSequenceOutput {
        format: formats::image(graph, format)?,
        start_number: common::non_negative_u32(Some(start))?,
    }))
}

pub(super) fn caption(
    graph: &FrozenDomainGraph,
    values: &[Value],
) -> Result<DeliverableKind, ExecutableLowerError> {
    let [_, _, format, layers] = values else {
        return Err(malformed());
    };
    let mut track_ids = value::list(Some(layers))?
        .iter()
        .map(|layer| common::track_ref(graph, layer))
        .collect::<Result<Vec<_>, _>>()?;
    track_ids.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    Ok(DeliverableKind::CaptionSidecar(CaptionSidecarOutput {
        format: formats::caption(graph, format)?,
        track_ids,
    }))
}

pub(super) fn stem(
    graph: &FrozenDomainGraph,
    values: &[Value],
) -> Result<DeliverableKind, ExecutableLowerError> {
    let [_, _, format, encoding, source] = values else {
        return Err(malformed());
    };
    Ok(DeliverableKind::AudioStem(AudioStemOutput {
        format: formats::stem(graph, format)?,
        audio: common::audio_output(graph, encoding)?,
        source: common::mix_source(graph, source)?,
    }))
}

pub(super) fn mp3(
    graph: &FrozenDomainGraph,
    values: &[Value],
) -> Result<DeliverableKind, ExecutableLowerError> {
    let [_, _, source, bitrate, sample_rate, channels] = values else {
        return Err(malformed());
    };
    Ok(DeliverableKind::AudioFile(AudioFile {
        source: common::mix_source(graph, source)?,
        encoding: AudioFileEncoding::Mp3(Mp3Encoding {
            bitrate_bps: common::positive_u32(Some(bitrate))?,
            sample_rate_hz: common::positive_u32(Some(sample_rate))?,
            channel_layout: common::channel_layout(graph, channels)?,
        }),
    }))
}

pub(super) fn scope(
    graph: &FrozenDomainGraph,
    values: &[Value],
    timebase: u32,
) -> Result<DeliverableKind, ExecutableLowerError> {
    let [_, _, scope, at, canvas, format] = values else {
        return Err(malformed());
    };
    let (width, height) = common::canvas(graph, canvas)?;
    Ok(DeliverableKind::Scope(ScopeOutput {
        scope: formats::scope(graph, scope)?,
        at: time::coordinate(Some(at), timebase)?,
        width,
        height,
        format: formats::image(graph, format)?,
    }))
}

pub(super) fn gif(
    graph: &FrozenDomainGraph,
    values: &[Value],
) -> Result<DeliverableKind, ExecutableLowerError> {
    let [_, _, playback, dither] = values else {
        return Err(malformed());
    };
    Ok(DeliverableKind::AnimatedImage(AnimatedImage::Gif(
        GifAnimation {
            playback: formats::gif_playback(graph, playback)?,
            dither: formats::gif_dither(graph, dither)?,
        },
    )))
}

pub(super) fn still(
    graph: &FrozenDomainGraph,
    values: &[Value],
    timebase: u32,
) -> Result<DeliverableKind, ExecutableLowerError> {
    let [_, _, at, format] = values else {
        return Err(malformed());
    };
    Ok(DeliverableKind::StillImage(StillImage {
        frame: FrameSelection::Containing {
            at: time::coordinate(Some(at), timebase)?,
        },
        encoding: formats::image(graph, format)?,
    }))
}

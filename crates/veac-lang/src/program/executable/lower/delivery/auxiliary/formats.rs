use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{
    AudioStemFormat, CaptionSidecarFormat, GifDither, GifPlayback, ImageFormat, VideoScope,
};

use super::super::super::value;
use super::super::{common, malformed, ExecutableLowerError};

pub(super) fn image(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<ImageFormat, ExecutableLowerError> {
    Ok(match common::empty(graph, source)? {
        Op::ImagePng => ImageFormat::Png,
        Op::ImageJpeg => ImageFormat::Jpeg,
        Op::ImageTiff => ImageFormat::Tiff,
        Op::ImageExr => ImageFormat::Exr,
        _ => return Err(malformed()),
    })
}

pub(super) fn caption(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<CaptionSidecarFormat, ExecutableLowerError> {
    match common::empty(graph, source)? {
        Op::CaptionSrt => Ok(CaptionSidecarFormat::Srt),
        Op::CaptionWebvtt => Ok(CaptionSidecarFormat::WebVtt),
        Op::CaptionAss => Ok(CaptionSidecarFormat::Ass),
        _ => Err(malformed()),
    }
}

pub(super) fn stem(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<AudioStemFormat, ExecutableLowerError> {
    match common::empty(graph, source)? {
        Op::StemWav => Ok(AudioStemFormat::Wav),
        Op::StemFlac => Ok(AudioStemFormat::Flac),
        _ => Err(malformed()),
    }
}

pub(super) fn scope(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<VideoScope, ExecutableLowerError> {
    match common::empty(graph, source)? {
        Op::ScopeWaveform => Ok(VideoScope::Waveform),
        Op::ScopeVectorscope => Ok(VideoScope::Vectorscope),
        Op::ScopeHistogram => Ok(VideoScope::Histogram),
        _ => Err(malformed()),
    }
}

pub(super) fn gif_playback(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<GifPlayback, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::GifOnce, []) => Ok(GifPlayback::Once),
        (Op::GifForever, []) => Ok(GifPlayback::Forever),
        (Op::GifTimes, [count]) => Ok(GifPlayback::Times {
            count: common::positive_u16(Some(count))?,
        }),
        _ => Err(malformed()),
    }
}

pub(super) fn gif_dither(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<GifDither, ExecutableLowerError> {
    Ok(match common::empty(graph, source)? {
        Op::GifDitherBayer => GifDither::Bayer,
        Op::GifDitherFloydSteinberg => GifDither::FloydSteinberg,
        Op::GifDitherSierra2 => GifDither::Sierra2,
        Op::GifDitherNone => GifDither::None,
        _ => return Err(malformed()),
    })
}

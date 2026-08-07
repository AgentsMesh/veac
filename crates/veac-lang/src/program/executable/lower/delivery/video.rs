use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{OutputFormat, VideoDeliverable, VideoOutput};

use super::super::value;
use super::{common, malformed, ExecutableLowerError};

mod codec;
mod options;

pub(super) fn color(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<veac_ir::ColorSpace>, ExecutableLowerError> {
    options::color(graph, source)
}

pub(super) fn b_frames(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<u8>, ExecutableLowerError> {
    options::b_frames(graph, source)
}

pub(super) fn delivery(
    graph: &FrozenDomainGraph,
    values: &[Value],
) -> Result<VideoDeliverable, ExecutableLowerError> {
    let [_, _, settings] = values else {
        return Err(malformed());
    };
    let values = value::description_operands(graph, settings, Op::VideoDelivery)?;
    let [container, video, audio, streaming, passes, hardware] = values else {
        return Err(malformed());
    };
    Ok(VideoDeliverable {
        container: container_value(graph, container)?,
        video: output(graph, video)?,
        audio: options::embedded_audio(graph, audio)?,
        optimize_for_streaming: value::boolean(Some(streaming))?,
        pass_mode: options::pass_mode(graph, passes)?,
        hardware: options::hardware(graph, hardware)?,
    })
}

fn output(graph: &FrozenDomainGraph, source: &Value) -> Result<VideoOutput, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::VideoOutput)?;
    let [codec_value, pixel, alpha, color, rate, gop, b_frames, profile, level] = values else {
        return Err(malformed());
    };
    let codec = codec::codec(graph, codec_value)?;
    Ok(VideoOutput {
        codec,
        pixel_format: codec::pixel(graph, pixel)?,
        alpha: codec::alpha(graph, alpha)?,
        color_space: options::color(graph, color)?,
        rate_control: options::rate(graph, rate)?,
        gop_size: options::gop(graph, gop)?,
        b_frames: options::b_frames(graph, b_frames)?,
        profile: codec::profile_choice(graph, profile)?,
        level: options::level_choice(graph, level, codec)?,
    })
}

fn container_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<OutputFormat, ExecutableLowerError> {
    Ok(match common::empty(graph, source)? {
        Op::ContainerMp4 => OutputFormat::Mp4,
        Op::ContainerMov => OutputFormat::Mov,
        Op::ContainerMkv => OutputFormat::Mkv,
        Op::ContainerWebm => OutputFormat::Webm,
        Op::ContainerMxf => OutputFormat::Mxf,
        _ => return Err(malformed()),
    })
}

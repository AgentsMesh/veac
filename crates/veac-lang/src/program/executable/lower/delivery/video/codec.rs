use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{AlphaMode, PixelFormat, VideoCodec, VideoProfile};

use super::super::super::value;
use super::super::{common, malformed, ExecutableLowerError};

pub(super) fn codec(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<VideoCodec, ExecutableLowerError> {
    Ok(match common::empty(graph, source)? {
        Op::VideoH264 => VideoCodec::H264,
        Op::VideoH265 => VideoCodec::H265,
        Op::VideoVp9 => VideoCodec::Vp9,
        Op::VideoAv1 => VideoCodec::Av1,
        Op::VideoProres => VideoCodec::ProRes,
        Op::VideoDnxhr => VideoCodec::DnxHr,
        _ => return Err(malformed()),
    })
}

pub(super) fn pixel(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<PixelFormat, ExecutableLowerError> {
    Ok(match common::empty(graph, source)? {
        Op::PixelYuv420p => PixelFormat::Yuv420p,
        Op::PixelYuv420p10le => PixelFormat::Yuv420p10le,
        Op::PixelYuv422p => PixelFormat::Yuv422p,
        Op::PixelYuv422p10le => PixelFormat::Yuv422p10le,
        Op::PixelYuv444p10le => PixelFormat::Yuv444p10le,
        Op::PixelYuva444p10le => PixelFormat::Yuva444p10le,
        _ => return Err(malformed()),
    })
}

pub(super) fn alpha(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<AlphaMode, ExecutableLowerError> {
    match common::empty(graph, source)? {
        Op::AlphaOpaque => Ok(AlphaMode::Opaque),
        Op::AlphaStraight => Ok(AlphaMode::Straight),
        _ => Err(malformed()),
    }
}

pub(super) fn profile_choice(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<VideoProfile>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::VideoProfileAuto, []) => Ok(None),
        (Op::VideoProfilePresent, [profile]) => Ok(Some(profile_value(graph, profile)?)),
        _ => Err(malformed()),
    }
}

fn profile_value(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<VideoProfile, ExecutableLowerError> {
    Ok(match common::empty(graph, source)? {
        Op::ProfileH264Baseline => VideoProfile::H264Baseline,
        Op::ProfileH264Main => VideoProfile::H264Main,
        Op::ProfileH264High => VideoProfile::H264High,
        Op::ProfileH264High10 => VideoProfile::H264High10,
        Op::ProfileH265Main => VideoProfile::H265Main,
        Op::ProfileH265Main10 => VideoProfile::H265Main10,
        Op::ProfileVp90 => VideoProfile::Vp9Profile0,
        Op::ProfileVp92 => VideoProfile::Vp9Profile2,
        Op::ProfileAv1Main => VideoProfile::Av1Main,
        Op::ProfileProres4444 => VideoProfile::ProRes4444,
        Op::ProfileDnxhrLb => VideoProfile::DnxHrLb,
        Op::ProfileDnxhrSq => VideoProfile::DnxHrSq,
        Op::ProfileDnxhrHq => VideoProfile::DnxHrHq,
        Op::ProfileDnxhrHqx => VideoProfile::DnxHrHqx,
        Op::ProfileDnxhr444 => VideoProfile::DnxHr444,
        _ => return Err(malformed()),
    })
}

use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{HardwareBackend, HardwareSelection, PassMode, VideoRateControl};

use super::super::super::{color, value};
use super::super::{common, malformed, ExecutableLowerError};

pub(super) fn embedded_audio(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<veac_ir::AudioOutput>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::EmbeddedAudioNone, []) => Ok(None),
        (Op::EmbeddedAudioPresent, [audio]) => Ok(Some(common::audio_output(graph, audio)?)),
        _ => Err(malformed()),
    }
}

pub(super) fn color(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<veac_ir::ColorSpace>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::VideoColorUnspecified, []) => Ok(None),
        (Op::VideoColorPresent, [space]) => Ok(Some(color::color_space(graph, space)?)),
        _ => Err(malformed()),
    }
}

pub(super) fn rate(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<VideoRateControl, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::VideoCrf, [amount]) => Ok(VideoRateControl::Crf {
            value: common::non_negative_u8(Some(amount))?,
        }),
        (Op::VideoBitrate, [target]) => Ok(VideoRateControl::Bitrate {
            target_bps: common::positive_u64(Some(target))?,
            max_bps: None,
            buffer_size_bits: None,
        }),
        (Op::VideoCappedBitrate, [target, max, buffer]) => Ok(VideoRateControl::Bitrate {
            target_bps: common::positive_u64(Some(target))?,
            max_bps: Some(common::positive_u64(Some(max))?),
            buffer_size_bits: Some(common::positive_u64(Some(buffer))?),
        }),
        (Op::VideoLossless, []) => Ok(VideoRateControl::Lossless),
        _ => Err(malformed()),
    }
}

pub(super) fn gop(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<u32>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::GopAuto, []) => Ok(None),
        (Op::GopFrames, [amount]) => Ok(Some(common::positive_u32(Some(amount))?)),
        _ => Err(malformed()),
    }
}

pub(super) fn b_frames(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<u8>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::BFramesAuto, []) => Ok(None),
        (Op::BFramesCount, [amount]) => Ok(Some(common::non_negative_u8(Some(amount))?)),
        _ => Err(malformed()),
    }
}

pub(super) fn level_choice(
    graph: &FrozenDomainGraph,
    source: &Value,
    codec: veac_ir::VideoCodec,
) -> Result<Option<String>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::VideoLevelAuto, []) => Ok(None),
        (Op::VideoLevelPresent, [level]) => Ok(Some(level_value(graph, level, codec)?)),
        _ => Err(malformed()),
    }
}

fn level_value(
    graph: &FrozenDomainGraph,
    source: &Value,
    codec: veac_ir::VideoCodec,
) -> Result<String, ExecutableLowerError> {
    let (operation, values) = value::description(graph, source)?;
    if !matches!(
        (codec, operation),
        (veac_ir::VideoCodec::H264, Op::H264Level)
            | (veac_ir::VideoCodec::H265, Op::H265Level)
            | (veac_ir::VideoCodec::Vp9, Op::Vp9Level)
            | (veac_ir::VideoCodec::Av1, Op::Av1Level)
    ) {
        return Err(malformed());
    }
    let [major, minor] = values else {
        return Err(malformed());
    };
    let major = common::non_negative_u32(Some(major))?;
    let minor = common::non_negative_u32(Some(minor))?;
    Ok(if minor == 0 {
        major.to_string()
    } else {
        format!("{major}.{minor}")
    })
}

pub(super) fn pass_mode(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<PassMode, ExecutableLowerError> {
    match common::empty(graph, source)? {
        Op::PassSingle => Ok(PassMode::Single),
        Op::PassTwo => Ok(PassMode::TwoPass),
        _ => Err(malformed()),
    }
}

pub(super) fn hardware(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<HardwareSelection, ExecutableLowerError> {
    Ok(match common::empty(graph, source)? {
        Op::HardwareAuto => HardwareSelection::Auto,
        Op::HardwareSoftware => HardwareSelection::Software,
        Op::HardwareVideotoolbox => explicit(HardwareBackend::VideoToolbox),
        Op::HardwareNvenc => explicit(HardwareBackend::Nvenc),
        Op::HardwareQsv => explicit(HardwareBackend::Qsv),
        Op::HardwareVaapi => explicit(HardwareBackend::Vaapi),
        _ => return Err(malformed()),
    })
}

fn explicit(backend: HardwareBackend) -> HardwareSelection {
    HardwareSelection::Explicit { backend }
}

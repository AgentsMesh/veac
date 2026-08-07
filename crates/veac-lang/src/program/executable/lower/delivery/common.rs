use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{
    AudioChannelLayout, AudioCodec, AudioMixSource, AudioOutput, CaptionOutput, Rational,
};

use super::super::{id, value};
use super::{malformed, ExecutableLowerError};

pub(super) fn canvas(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<(u32, u32), ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::Canvas)?;
    let [width, height] = values else {
        return Err(malformed());
    };
    Ok((dimension(Some(width))?, dimension(Some(height))?))
}

pub(super) fn frame_rate(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Rational, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::FrameRate)?;
    let [numerator, denominator] = values else {
        return Err(malformed());
    };
    malformed_result(Rational::new(
        positive_i64(Some(numerator))?,
        positive_u32(Some(denominator))?,
    ))
}

pub(super) fn caption_output(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<CaptionOutput, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::CaptionBurnIn, []) => Ok(CaptionOutput::BurnIn),
        (Op::CaptionDiscard, []) => Ok(CaptionOutput::Discard),
        _ => Err(malformed()),
    }
}

pub(super) fn audio_output(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<AudioOutput, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::AudioOutput)?;
    let [codec, sample_rate, channels] = values else {
        return Err(malformed());
    };
    Ok(AudioOutput {
        codec: audio_codec(graph, codec)?,
        sample_rate: positive_u32(Some(sample_rate))?,
        channels: positive_u8(Some(channels))?,
    })
}

pub(super) fn audio_codec(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<AudioCodec, ExecutableLowerError> {
    Ok(match empty(graph, source)? {
        Op::AudioAac => AudioCodec::Aac,
        Op::AudioOpus => AudioCodec::Opus,
        Op::AudioFlac => AudioCodec::Flac,
        Op::AudioPcmS16le => AudioCodec::PcmS16Le,
        Op::AudioPcmS24le => AudioCodec::PcmS24Le,
        Op::AudioPcmS32le => AudioCodec::PcmS32Le,
        _ => return Err(malformed()),
    })
}

pub(super) fn channel_layout(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<AudioChannelLayout, ExecutableLowerError> {
    match empty(graph, source)? {
        Op::ChannelMono => Ok(AudioChannelLayout::Mono),
        Op::ChannelStereo => Ok(AudioChannelLayout::Stereo),
        _ => Err(malformed()),
    }
}

pub(super) fn mix_source(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<AudioMixSource, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::MixMaster, []) => Ok(AudioMixSource::Master),
        (Op::MixLayer, [layer]) => Ok(AudioMixSource::Track {
            track_id: id::track(&logical_ref(graph, layer)?),
        }),
        (Op::MixBus, [bus]) => {
            let values = value::description_operands(graph, bus, Op::AudioBus)?;
            Ok(AudioMixSource::Bus {
                bus_id: malformed_result(veac_ir::BusId::new(format!(
                    "bus_{}",
                    value::identifier(values.first())?
                )))?,
            })
        }
        _ => Err(malformed()),
    }
}

pub(super) fn sequence_ref(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::SequenceId, ExecutableLowerError> {
    Ok(id::sequence(&logical_ref(graph, source)?))
}

pub(super) fn track_ref(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<veac_ir::TrackId, ExecutableLowerError> {
    Ok(id::track(&logical_ref(graph, source)?))
}

fn logical_ref<'a>(
    graph: &'a FrozenDomainGraph,
    source: &Value,
) -> Result<Vec<&'a str>, ExecutableLowerError> {
    let Value::Domain(handle) = source else {
        return Err(malformed());
    };
    graph.logical_key(handle).ok_or_else(malformed)
}

pub(super) fn positive_u64(value: Option<&Value>) -> Result<u64, ExecutableLowerError> {
    malformed_result(u64::try_from(positive_i64(value)?))
}

pub(super) fn positive_u32(value: Option<&Value>) -> Result<u32, ExecutableLowerError> {
    malformed_result(u32::try_from(positive_i64(value)?))
}

pub(super) fn positive_u16(value: Option<&Value>) -> Result<u16, ExecutableLowerError> {
    malformed_result(u16::try_from(positive_i64(value)?))
}

pub(super) fn positive_u8(value: Option<&Value>) -> Result<u8, ExecutableLowerError> {
    malformed_result(u8::try_from(positive_i64(value)?))
}

pub(super) fn non_negative_u32(value: Option<&Value>) -> Result<u32, ExecutableLowerError> {
    malformed_result(u32::try_from(value::integer(value)?))
}

pub(super) fn non_negative_u8(value: Option<&Value>) -> Result<u8, ExecutableLowerError> {
    malformed_result(u8::try_from(value::integer(value)?))
}

fn positive_i64(value: Option<&Value>) -> Result<i64, ExecutableLowerError> {
    let value = value::integer(value)?;
    (value > 0).then_some(value).ok_or_else(malformed)
}

fn dimension(value: Option<&Value>) -> Result<u32, ExecutableLowerError> {
    let exact = value::exact(value)?;
    if exact.numerator() <= 0 || exact.denominator() != 1 {
        return Err(malformed());
    }
    malformed_result(u32::try_from(exact.numerator()))
}

pub(super) fn empty(graph: &FrozenDomainGraph, source: &Value) -> Result<Op, ExecutableLowerError> {
    let (operation, values) = value::description(graph, source)?;
    values.is_empty().then_some(operation).ok_or_else(malformed)
}

fn malformed_result<T, E>(result: Result<T, E>) -> Result<T, ExecutableLowerError> {
    match result {
        Ok(value) => Ok(value),
        Err(_) => Err(malformed()),
    }
}

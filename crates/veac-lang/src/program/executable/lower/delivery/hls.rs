use crate::program::expression::runtime::domain_graph::FrozenDomainGraph;
use crate::program::expression::Value;
use crate::program::DomainOperationId as Op;
use veac_ir::{
    AacEncoding, AdaptivePackage, DeliverableKind, HlsAudio, HlsAudioEncoding, HlsCappedBitrate,
    HlsH264Encoding, HlsH264Profile, HlsPackage, HlsRendition, HlsRenditionRaster,
    HlsVideoEncoding,
};

use super::super::{id, time, value};
use super::{common, malformed, video, ExecutableLowerError};

pub(super) fn package(
    graph: &FrozenDomainGraph,
    values: &[Value],
    timebase: u32,
    path: &[&str],
) -> Result<DeliverableKind, ExecutableLowerError> {
    let [_, _, segment, audio, renditions] = values else {
        return Err(malformed());
    };
    let mut renditions = value::list(Some(renditions))?
        .iter()
        .map(|source| rendition(graph, source, path))
        .collect::<Result<Vec<_>, _>>()?;
    renditions.sort_by(|left, right| left.id.as_str().cmp(right.id.as_str()));
    Ok(DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(
        HlsPackage {
            segment_duration: time::coordinate(Some(segment), timebase)?,
            audio: audio_choice(graph, audio)?,
            renditions,
        },
    )))
}

fn audio_choice(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<HlsAudio>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::HlsAudioNone, []) => Ok(None),
        (Op::HlsAudioAac, [source, bitrate, sample_rate, channels]) => Ok(Some(HlsAudio {
            source: common::mix_source(graph, source)?,
            encoding: HlsAudioEncoding::Aac(AacEncoding {
                bitrate_bps: common::positive_u32(Some(bitrate))?,
                sample_rate_hz: common::positive_u32(Some(sample_rate))?,
                channel_layout: common::channel_layout(graph, channels)?,
            }),
        })),
        _ => Err(malformed()),
    }
}

fn rendition(
    graph: &FrozenDomainGraph,
    source: &Value,
    package_path: &[&str],
) -> Result<HlsRendition, ExecutableLowerError> {
    let values = value::description_operands(graph, source, Op::HlsRendition)?;
    let [key, canvas, target, max, buffer, profile, level, color, b_frames] = values else {
        return Err(malformed());
    };
    let mut path = package_path.to_vec();
    path.push(value::identifier(Some(key))?);
    let (width, height) = common::canvas(graph, canvas)?;
    Ok(HlsRendition {
        id: id::rendition(&path),
        raster: HlsRenditionRaster { width, height },
        encoding: HlsVideoEncoding::H264(HlsH264Encoding {
            rate_control: HlsCappedBitrate {
                target_bps: common::positive_u64(Some(target))?,
                max_bps: common::positive_u64(Some(max))?,
                buffer_size_bits: common::positive_u64(Some(buffer))?,
            },
            profile: profile_choice(graph, profile)?,
            level: level_choice(graph, level)?,
            color_space: video::color(graph, color)?,
            b_frames: video::b_frames(graph, b_frames)?,
        }),
    })
}

fn profile_choice(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<HlsH264Profile>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::HlsProfileAuto, []) => Ok(None),
        (Op::HlsProfilePresent, [profile]) => Ok(Some(match common::empty(graph, profile)? {
            Op::HlsProfileBaseline => HlsH264Profile::Baseline,
            Op::HlsProfileMain => HlsH264Profile::Main,
            Op::HlsProfileHigh => HlsH264Profile::High,
            _ => return Err(malformed()),
        })),
        _ => Err(malformed()),
    }
}

fn level_choice(
    graph: &FrozenDomainGraph,
    source: &Value,
) -> Result<Option<String>, ExecutableLowerError> {
    match value::description(graph, source)? {
        (Op::VideoLevelAuto, []) => Ok(None),
        (Op::VideoLevelPresent, [level]) => {
            let values = value::description_operands(graph, level, Op::H264Level)?;
            let [major, minor] = values else {
                return Err(malformed());
            };
            let major = common::non_negative_u32(Some(major))?;
            let minor = common::non_negative_u32(Some(minor))?;
            Ok(Some(if minor == 0 {
                major.to_string()
            } else {
                format!("{major}.{minor}")
            }))
        }
        _ => Err(malformed()),
    }
}

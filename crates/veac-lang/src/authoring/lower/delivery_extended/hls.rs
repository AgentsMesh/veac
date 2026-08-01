use crate::authoring::{
    AdaptivePackageRecipe, HlsAudioDecl, HlsH264EncodingDecl, HlsH264ProfileDecl, HlsRenditionDecl,
};
use veac_ir::{
    AacEncoding, AdaptivePackage, DeliverableKind, HlsAudio, HlsAudioEncoding, HlsCappedBitrate,
    HlsH264Encoding, HlsH264Profile, HlsPackage, HlsRendition, HlsRenditionRaster,
    HlsVideoEncoding,
};

use super::super::{
    context::Context, delivery::stem_source, delivery_units, ids, output_types_aux,
};

pub(super) fn lower(ctx: &mut Context, value: &AdaptivePackageRecipe) -> Option<DeliverableKind> {
    let mut renditions = value
        .renditions
        .iter()
        .map(|value| rendition(ctx, value))
        .collect::<Option<Vec<_>>>()?;
    renditions.sort_by(|left, right| left.id.cmp(&right.id));
    Some(DeliverableKind::AdaptivePackage(AdaptivePackage::Hls(
        HlsPackage {
            segment_duration: super::super::value::time(ctx, &value.segment_duration)?,
            audio: match &value.audio {
                Some(value) => Some(audio(ctx, value)?),
                None => None,
            },
            renditions,
        },
    )))
}

fn audio(ctx: &mut Context, value: &HlsAudioDecl) -> Option<HlsAudio> {
    Some(HlsAudio {
        source: stem_source(ctx, &value.source)?,
        encoding: HlsAudioEncoding::Aac(AacEncoding {
            bitrate_bps: delivery_units::bitrate_u32(ctx, &value.bitrate, "AAC bitrate")?,
            sample_rate_hz: delivery_units::sample_rate(
                ctx,
                &value.sample_rate,
                "AAC sample-rate",
            )?,
            channel_layout: super::channel_layout(value.channel_layout),
        }),
    })
}

fn rendition(ctx: &mut Context, value: &HlsRenditionDecl) -> Option<HlsRendition> {
    Some(HlsRendition {
        id: ids::hls_rendition(ctx, &value.id)?,
        raster: HlsRenditionRaster {
            width: delivery_units::pixels(ctx, &value.width, "HLS rendition width")?,
            height: delivery_units::pixels(ctx, &value.height, "HLS rendition height")?,
        },
        encoding: HlsVideoEncoding::H264(h264(ctx, &value.encoding)?),
    })
}

fn h264(ctx: &mut Context, value: &HlsH264EncodingDecl) -> Option<HlsH264Encoding> {
    Some(HlsH264Encoding {
        rate_control: HlsCappedBitrate {
            target_bps: delivery_units::bitrate(
                ctx,
                &value.rate_control.target,
                "HLS target bitrate",
            )?,
            max_bps: delivery_units::bitrate(ctx, &value.rate_control.max, "HLS maximum bitrate")?,
            buffer_size_bits: delivery_units::buffer_size(
                ctx,
                &value.rate_control.buffer,
                "HLS rate-control buffer",
            )?,
        },
        profile: value.profile.map(|value| match value {
            HlsH264ProfileDecl::Baseline => HlsH264Profile::Baseline,
            HlsH264ProfileDecl::Main => HlsH264Profile::Main,
            HlsH264ProfileDecl::High => HlsH264Profile::High,
        }),
        level: value.level.clone(),
        color_space: value.color_space.map(output_types_aux::color_space),
        b_frames: value.b_frames,
    })
}

use crate::authoring::{
    AdaptivePackageRecipe, HlsH264EncodingDecl, HlsH264ProfileDecl, HlsRenditionDecl,
};

use super::super::output_aux::mix_source;
use super::super::output_video::color_space;
use super::super::writer::Writer;

pub(super) fn package(writer: &mut Writer, value: &AdaptivePackageRecipe) {
    writer.block("package hls", |writer| {
        writer.line(format!("segment-duration {};", value.segment_duration.raw));
        match &value.audio {
            Some(audio) => writer.block("audio", |writer| {
                writer.line(format!("source {};", mix_source(&audio.source)));
                writer.block("encode aac", |writer| {
                    writer.line(format!("bitrate {};", audio.bitrate.raw));
                    writer.line(format!("sample-rate {};", audio.sample_rate.raw));
                    writer.line(format!(
                        "channel-layout {};",
                        super::channel_layout(audio.channel_layout)
                    ));
                });
            }),
            None => writer.line("audio none;"),
        }
        for rendition in &value.renditions {
            rendition_block(writer, rendition);
        }
    });
}

fn rendition_block(writer: &mut Writer, value: &HlsRenditionDecl) {
    writer.block(format!("rendition {}", value.id.value), |writer| {
        writer.line(format!(
            "canvas {} by {};",
            value.width.raw, value.height.raw
        ));
        writer.block("encode h264", |writer| h264(writer, &value.encoding));
    });
}

fn h264(writer: &mut Writer, value: &HlsH264EncodingDecl) {
    writer.block("rate-control capped", |writer| {
        writer.line(format!("target {};", value.rate_control.target.raw));
        writer.line(format!("max {};", value.rate_control.max.raw));
        writer.line(format!("buffer {};", value.rate_control.buffer.raw));
    });
    writer.line(match value.profile {
        Some(value) => format!("profile {};", profile(value)),
        None => "profile automatic;".into(),
    });
    writer.line(match &value.level {
        Some(value) => format!("level {};", super::super::value::quoted(value)),
        None => "level automatic;".into(),
    });
    match &value.color_space {
        Some(value) => writer.block("color-space", |writer| color_space(writer, value)),
        None => writer.line("color-space source;"),
    }
    writer.line(match value.b_frames {
        Some(value) => format!("b-frames {value};"),
        None => "b-frames automatic;".into(),
    });
}

fn profile(value: HlsH264ProfileDecl) -> &'static str {
    match value {
        HlsH264ProfileDecl::Baseline => "baseline",
        HlsH264ProfileDecl::Main => "main",
        HlsH264ProfileDecl::High => "high",
    }
}

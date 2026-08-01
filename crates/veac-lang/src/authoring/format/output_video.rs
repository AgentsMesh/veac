use crate::authoring::{
    AudioOutput, ColorSpace, HardwareSelection, OutputKeyword, VideoEncoding, VideoOutput,
    VideoRateControl,
};

use super::output_units::{bitrate, buffer_size, channel_layout, sample_rate};
use super::writer::Writer;

pub(super) fn video(writer: &mut Writer, value: &VideoEncoding) {
    writer.block(format!("mux {}", value.container.token()), |writer| {
        writer.line(format!(
            "layout {};",
            if value.optimize_for_streaming {
                "fast-start"
            } else {
                "standard"
            }
        ));
        writer.block(format!("video {}", value.video.codec.token()), |writer| {
            video_settings(writer, &value.video)
        });
        match &value.audio {
            Some(value) => writer.block(format!("audio {}", value.codec.token()), |writer| {
                audio_settings(writer, value)
            }),
            None => writer.line("audio none;"),
        }
        writer.line(format!("passes {};", value.pass_mode.token()));
        writer.line(format!("accelerator {};", hardware(&value.hardware)));
    });
}

pub(super) fn video_settings(writer: &mut Writer, value: &VideoOutput) {
    writer.line(format!("pixel-format {};", value.pixel_format.token()));
    writer.line(format!("alpha {};", value.alpha.token()));
    match &value.color_space {
        Some(value) => writer.block("color-space", |writer| color_space(writer, value)),
        None => writer.line("color-space source;"),
    }
    rate_control(writer, value.rate_control.clone());
    writer.line(match value.gop_size {
        Some(value) => format!("gop {value};"),
        None => "gop automatic;".into(),
    });
    writer.line(match value.b_frames {
        Some(value) => format!("b-frames {value};"),
        None => "b-frames automatic;".into(),
    });
    writer.line(match &value.profile {
        Some(value) => format!("profile {};", value.token()),
        None => "profile automatic;".into(),
    });
    writer.line(match &value.level {
        Some(value) => format!("level {};", super::value::quoted(value)),
        None => "level automatic;".into(),
    });
}

pub(super) fn audio_settings(writer: &mut Writer, value: &AudioOutput) {
    writer.line(format!("sample-rate {};", sample_rate(value.sample_rate)));
    writer.line(format!(
        "channel-layout {};",
        channel_layout(value.channels)
    ));
}

fn rate_control(writer: &mut Writer, value: VideoRateControl) {
    match value {
        VideoRateControl::Crf { value } => writer.block("rate-control crf", |writer| {
            writer.line(format!("value {value};"));
        }),
        VideoRateControl::Bitrate {
            target_bps,
            max_bps: Some(max_bps),
            buffer_size_bits: Some(buffer),
        } => writer.block("rate-control capped", |writer| {
            writer.line(format!("target {};", bitrate(target_bps)));
            writer.line(format!("max {};", bitrate(max_bps)));
            writer.line(format!("buffer {};", buffer_size(buffer)));
        }),
        VideoRateControl::Bitrate { target_bps, .. } => {
            writer.block("rate-control average", |writer| {
                writer.line(format!("target {};", bitrate(target_bps)));
            })
        }
        VideoRateControl::Lossless => writer.line("rate-control lossless;"),
    }
}

pub(super) fn color_space(writer: &mut Writer, value: &ColorSpace) {
    writer.line(format!("primaries {};", value.primaries.token()));
    writer.line(format!("transfer {};", value.transfer.token()));
    writer.line(format!("matrix {};", value.matrix.token()));
    writer.line(format!("range {};", value.range.token()));
}

fn hardware(value: &HardwareSelection) -> &'static str {
    match value {
        HardwareSelection::Auto => "auto",
        HardwareSelection::Software => "software",
        HardwareSelection::Explicit { backend } => backend.token(),
    }
}

use crate::authoring::{
    AudioOutput, ColorSpace, HardwareSelection, OutputKeyword, VideoEncoding, VideoOutput,
    VideoRateControl,
};

use super::writer::Writer;

pub(super) fn video(writer: &mut Writer, value: &VideoEncoding) {
    writer.line(format!("container {};", value.container.token()));
    writer.block("video", |writer| video_settings(writer, &value.video));
    match &value.audio {
        Some(value) => writer.block("audio", |writer| audio_settings(writer, value)),
        None => writer.line("audio none;"),
    }
    writer.line(format!("captions {};", value.captions.token()));
    writer.line(format!(
        "optimize-for-streaming {};",
        value.optimize_for_streaming
    ));
    writer.line(format!("pass-mode {};", value.pass_mode.token()));
    writer.line(format!("hardware {};", hardware(&value.hardware)));
}

fn video_settings(writer: &mut Writer, value: &VideoOutput) {
    writer.line(format!("codec {};", value.codec.token()));
    writer.line(format!("pixel-format {};", value.pixel_format.token()));
    writer.line(format!("alpha {};", value.alpha.token()));
    if let Some(value) = &value.color_space {
        writer.block("color-space", |writer| color_space(writer, value));
    }
    rate_control(writer, value.rate_control.clone());
    if let Some(value) = value.gop_size {
        writer.line(format!("gop-size {value};"));
    }
    if let Some(value) = value.b_frames {
        writer.line(format!("b-frames {value};"));
    }
    if let Some(value) = &value.profile {
        writer.line(format!("profile {};", value.token()));
    }
    if let Some(value) = &value.level {
        writer.line(format!("level {};", super::value::quoted(value)));
    }
}

fn audio_settings(writer: &mut Writer, value: &AudioOutput) {
    writer.line(format!("codec {};", value.codec.token()));
    writer.line(format!("sample-rate {};", value.sample_rate));
    writer.line(format!("channels {};", value.channels));
}

fn rate_control(writer: &mut Writer, value: VideoRateControl) {
    match value {
        VideoRateControl::Crf { value } => writer.block("rate-control crf", |writer| {
            writer.line(format!("value {value};"));
        }),
        VideoRateControl::Bitrate {
            target_bps,
            max_bps,
            buffer_bps,
        } => writer.block("rate-control bitrate", |writer| {
            writer.line(format!("target-bps {target_bps};"));
            if let Some(value) = max_bps {
                writer.line(format!("max-bps {value};"));
            }
            if let Some(value) = buffer_bps {
                writer.line(format!("buffer-bps {value};"));
            }
        }),
        VideoRateControl::Lossless => writer.line("rate-control lossless;"),
    }
}

fn color_space(writer: &mut Writer, value: &ColorSpace) {
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

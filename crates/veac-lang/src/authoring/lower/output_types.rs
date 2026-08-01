use crate::authoring as authored;
use veac_ir as canonical;

use super::output_types_aux as aux;

pub(super) fn video(value: &authored::VideoEncoding) -> canonical::VideoDeliverable {
    canonical::VideoDeliverable {
        container: container(value.container),
        video: video_settings(&value.video),
        audio: value.audio.as_ref().map(audio),
        optimize_for_streaming: value.optimize_for_streaming,
        pass_mode: match value.pass_mode {
            authored::PassMode::Single => canonical::PassMode::Single,
            authored::PassMode::TwoPass => canonical::PassMode::TwoPass,
        },
        hardware: aux::hardware(value.hardware),
    }
}

pub(super) fn caption_output(value: authored::CaptionOutput) -> canonical::CaptionOutput {
    match value {
        authored::CaptionOutput::BurnIn => canonical::CaptionOutput::BurnIn,
        authored::CaptionOutput::Discard => canonical::CaptionOutput::Discard,
    }
}

pub(super) fn video_settings(value: &authored::VideoOutput) -> canonical::VideoOutput {
    canonical::VideoOutput {
        codec: video_codec(value.codec),
        pixel_format: pixel_format(value.pixel_format),
        alpha: match value.alpha {
            authored::AlphaMode::Opaque => canonical::AlphaMode::Opaque,
            authored::AlphaMode::Straight => canonical::AlphaMode::Straight,
        },
        color_space: value.color_space.map(aux::color_space),
        rate_control: rate_control(&value.rate_control),
        gop_size: value.gop_size,
        b_frames: value.b_frames,
        profile: value.profile.map(aux::profile),
        level: value.level.clone(),
    }
}

pub(super) fn audio(value: &authored::AudioOutput) -> canonical::AudioOutput {
    canonical::AudioOutput {
        codec: match value.codec {
            authored::AudioCodec::Aac => canonical::AudioCodec::Aac,
            authored::AudioCodec::Opus => canonical::AudioCodec::Opus,
            authored::AudioCodec::PcmS16Le => canonical::AudioCodec::PcmS16Le,
            authored::AudioCodec::PcmS24Le => canonical::AudioCodec::PcmS24Le,
            authored::AudioCodec::PcmS32Le => canonical::AudioCodec::PcmS32Le,
            authored::AudioCodec::Flac => canonical::AudioCodec::Flac,
        },
        sample_rate: value.sample_rate,
        channels: value.channels,
    }
}

fn container(value: authored::OutputFormat) -> canonical::OutputFormat {
    match value {
        authored::OutputFormat::Mp4 => canonical::OutputFormat::Mp4,
        authored::OutputFormat::Mov => canonical::OutputFormat::Mov,
        authored::OutputFormat::Mkv => canonical::OutputFormat::Mkv,
        authored::OutputFormat::Webm => canonical::OutputFormat::Webm,
        authored::OutputFormat::Mxf => canonical::OutputFormat::Mxf,
    }
}

fn video_codec(value: authored::VideoCodec) -> canonical::VideoCodec {
    match value {
        authored::VideoCodec::H264 => canonical::VideoCodec::H264,
        authored::VideoCodec::H265 => canonical::VideoCodec::H265,
        authored::VideoCodec::Vp9 => canonical::VideoCodec::Vp9,
        authored::VideoCodec::Av1 => canonical::VideoCodec::Av1,
        authored::VideoCodec::ProRes => canonical::VideoCodec::ProRes,
        authored::VideoCodec::DnxHr => canonical::VideoCodec::DnxHr,
    }
}

fn pixel_format(value: authored::PixelFormat) -> canonical::PixelFormat {
    use authored::PixelFormat as A;
    use canonical::PixelFormat as C;
    match value {
        A::Yuv420p => C::Yuv420p,
        A::Yuv422p => C::Yuv422p,
        A::Yuv420p10le => C::Yuv420p10le,
        A::Yuv422p10le => C::Yuv422p10le,
        A::Yuv444p10le => C::Yuv444p10le,
        A::Yuva444p10le => C::Yuva444p10le,
    }
}

fn rate_control(value: &authored::VideoRateControl) -> canonical::VideoRateControl {
    match value {
        authored::VideoRateControl::Crf { value } => {
            canonical::VideoRateControl::Crf { value: *value }
        }
        authored::VideoRateControl::Bitrate {
            target_bps,
            max_bps,
            buffer_size_bits,
        } => canonical::VideoRateControl::Bitrate {
            target_bps: *target_bps,
            max_bps: *max_bps,
            buffer_size_bits: *buffer_size_bits,
        },
        authored::VideoRateControl::Lossless => canonical::VideoRateControl::Lossless,
    }
}

pub(super) fn image_format(value: authored::ImageFormat) -> canonical::ImageFormat {
    aux::image_format(value)
}

pub(super) fn caption_format(
    value: authored::CaptionSidecarFormat,
) -> canonical::CaptionSidecarFormat {
    aux::caption_format(value)
}

pub(super) fn stem_format(value: authored::AudioStemFormat) -> canonical::AudioStemFormat {
    aux::stem_format(value)
}

pub(super) fn scope(value: authored::VideoScope) -> canonical::VideoScope {
    aux::scope(value)
}

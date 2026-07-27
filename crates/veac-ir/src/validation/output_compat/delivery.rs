use crate::*;

pub fn video_delivery_valid(value: &VideoDeliverable) -> bool {
    let passes = match value.pass_mode {
        PassMode::Single => true,
        PassMode::TwoPass => {
            matches!(value.video.codec, VideoCodec::H264 | VideoCodec::H265)
                && matches!(value.video.rate_control, VideoRateControl::Bitrate { .. })
                && value.hardware == HardwareSelection::Software
        }
    };
    let native_rate = !matches!(value.video.codec, VideoCodec::ProRes | VideoCodec::DnxHr)
        || matches!(value.video.rate_control, VideoRateControl::Lossless);
    passes
        && native_rate
        && hardware_valid(value.video.codec, value.hardware)
        && encoder_level_valid(&value.video)
        && prores_valid(value)
        && dnxhr_valid(value)
        && alpha_container_valid(value)
        && color_delivery_valid(value)
}

fn encoder_level_valid(value: &VideoOutput) -> bool {
    value.level.is_none() || !matches!(value.codec, VideoCodec::H265 | VideoCodec::Av1)
}

fn prores_valid(value: &VideoDeliverable) -> bool {
    value.video.codec != VideoCodec::ProRes
        || (value.container == OutputFormat::Mov
            && value.video.profile == Some(VideoProfile::ProRes4444)
            && value.video.pixel_format == PixelFormat::Yuva444p10le
            && value.video.alpha == AlphaMode::Straight)
}

pub fn mxf_geometry_valid(width: u32, height: u32, rate: Rational) -> bool {
    let supported_rate = matches!(
        (rate.numerator, rate.denominator),
        (24 | 25 | 30 | 50 | 60, 1) | (24_000 | 30_000 | 60_000, 1_001)
    );
    width >= 256 && height >= 120 && supported_rate
}

fn dnxhr_valid(value: &VideoDeliverable) -> bool {
    if value.video.codec != VideoCodec::DnxHr {
        return value.container != OutputFormat::Mxf;
    }
    value.container == OutputFormat::Mxf
        && value.video.gop_size.is_none()
        && value.video.b_frames.is_none()
        && value.video.level.is_none()
        && value.pass_mode == PassMode::Single
        && value.audio.as_ref().is_none_or(|audio| {
            audio.sample_rate == 48_000
                && matches!(audio.codec, AudioCodec::PcmS16Le | AudioCodec::PcmS24Le)
        })
}

fn alpha_container_valid(value: &VideoDeliverable) -> bool {
    value.video.alpha == AlphaMode::Opaque
        || (value.container == OutputFormat::Mov
            && value.video.profile == Some(VideoProfile::ProRes4444))
}

fn color_delivery_valid(value: &VideoDeliverable) -> bool {
    let Some(color) = value.video.color_space else {
        return true;
    };
    let wide = color.primaries == ColorPrimaries::Bt2020;
    let hdr = matches!(
        color.transfer,
        ColorTransfer::Smpte2084 | ColorTransfer::AribStdB67
    );
    (!wide && !hdr)
        || (wide
            && color.matrix == ColorMatrix::Bt2020Ncl
            && value.video.pixel_format == PixelFormat::Yuv420p10le
            && matches!(value.video.codec, VideoCodec::H265 | VideoCodec::Av1))
}

fn hardware_valid(_: VideoCodec, selection: HardwareSelection) -> bool {
    matches!(
        selection,
        HardwareSelection::Auto | HardwareSelection::Software
    )
}

use veac_plan::canonical::{
    AlphaMode, HlsAudioEncoding, HlsH264Profile, HlsVideoEncoding, PixelFormat, VideoCodec,
    VideoOutput, VideoProfile, VideoRateControl,
};

use super::audio::AudioRenderSpec;

pub(super) fn video(value: &HlsVideoEncoding) -> VideoOutput {
    let HlsVideoEncoding::H264(value) = value;
    VideoOutput {
        codec: VideoCodec::H264,
        pixel_format: PixelFormat::Yuv420p,
        alpha: AlphaMode::Opaque,
        color_space: value.color_space,
        rate_control: VideoRateControl::Bitrate {
            target_bps: value.rate_control.target_bps,
            max_bps: Some(value.rate_control.max_bps),
            buffer_size_bits: Some(value.rate_control.buffer_size_bits),
        },
        gop_size: None,
        b_frames: value.b_frames,
        profile: value.profile.map(profile),
        level: value.level.clone(),
    }
}

pub(super) fn audio(value: &HlsAudioEncoding) -> AudioRenderSpec {
    let HlsAudioEncoding::Aac(value) = value;
    AudioRenderSpec {
        sample_rate: value.sample_rate_hz,
        channels: value.channel_layout.count(),
    }
}

fn profile(value: HlsH264Profile) -> VideoProfile {
    match value {
        HlsH264Profile::Baseline => VideoProfile::H264Baseline,
        HlsH264Profile::Main => VideoProfile::H264Main,
        HlsH264Profile::High => VideoProfile::H264High,
    }
}

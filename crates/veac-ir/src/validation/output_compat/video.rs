use crate::*;

/// Common compressed-video ceiling for the default untrusted-plan budget.
pub const MAX_VIDEO_BITRATE: u64 = 1_000_000_000;
/// Common rate-control buffer ceiling for the default untrusted-plan budget.
pub const MAX_VIDEO_BUFFER: u64 = 2_000_000_000;

pub fn video_settings_valid(video: &VideoOutput) -> bool {
    let rate_valid = match video.rate_control {
        VideoRateControl::Crf { value } => max_crf(video.codec).is_some_and(|max| value <= max),
        VideoRateControl::Bitrate {
            target_bps,
            max_bps,
            buffer_size_bits,
        } => {
            let complete_cap = max_bps.is_some() == buffer_size_bits.is_some();
            target_bps > 0
                && target_bps <= MAX_VIDEO_BITRATE
                && complete_cap
                && max_bps.is_none_or(|value| value >= target_bps && value <= MAX_VIDEO_BITRATE)
                && buffer_size_bits.is_none_or(|value| value > 0 && value <= MAX_VIDEO_BUFFER)
        }
        VideoRateControl::Lossless => true,
    };
    rate_valid
        && video
            .gop_size
            .is_none_or(|value| value > 0 && value <= i32::MAX as u32)
        && video.b_frames.is_none_or(|value| value <= 16)
        && !(video.profile == Some(VideoProfile::H264Baseline)
            && video.b_frames.is_some_and(|value| value > 0))
        && video
            .profile
            .is_none_or(|value| profile_codec(value) == video.codec)
        && pixel_profile_valid(video)
        && alpha_valid(video)
        && video
            .level
            .as_ref()
            .is_none_or(|value| super::level::valid(video.codec, value))
}

fn max_crf(codec: VideoCodec) -> Option<u8> {
    match codec {
        VideoCodec::H264 | VideoCodec::H265 => Some(51),
        VideoCodec::Vp9 | VideoCodec::Av1 => Some(63),
        VideoCodec::ProRes | VideoCodec::DnxHr => None,
    }
}

fn profile_codec(profile: VideoProfile) -> VideoCodec {
    match profile {
        VideoProfile::H264Baseline
        | VideoProfile::H264Main
        | VideoProfile::H264High
        | VideoProfile::H264High10 => VideoCodec::H264,
        VideoProfile::H265Main | VideoProfile::H265Main10 => VideoCodec::H265,
        VideoProfile::Vp9Profile0 | VideoProfile::Vp9Profile2 => VideoCodec::Vp9,
        VideoProfile::Av1Main => VideoCodec::Av1,
        VideoProfile::ProRes4444 => VideoCodec::ProRes,
        VideoProfile::DnxHrLb
        | VideoProfile::DnxHrSq
        | VideoProfile::DnxHrHq
        | VideoProfile::DnxHrHqx
        | VideoProfile::DnxHr444 => VideoCodec::DnxHr,
    }
}

fn pixel_profile_valid(video: &VideoOutput) -> bool {
    if video.codec == VideoCodec::ProRes {
        return matches!(
            (video.pixel_format, video.profile, video.alpha),
            (
                PixelFormat::Yuva444p10le,
                Some(VideoProfile::ProRes4444),
                AlphaMode::Straight
            )
        );
    }
    if video.codec == VideoCodec::DnxHr {
        return matches!(
            (video.pixel_format, video.profile),
            (
                PixelFormat::Yuv422p,
                Some(VideoProfile::DnxHrLb | VideoProfile::DnxHrSq | VideoProfile::DnxHrHq)
            ) | (PixelFormat::Yuv422p10le, Some(VideoProfile::DnxHrHqx))
                | (PixelFormat::Yuv444p10le, Some(VideoProfile::DnxHr444))
        );
    }
    if matches!(
        video.pixel_format,
        PixelFormat::Yuv422p | PixelFormat::Yuv422p10le | PixelFormat::Yuv444p10le
    ) {
        return false;
    }
    !matches!(
        (video.pixel_format, video.profile),
        (
            PixelFormat::Yuv420p10le,
            Some(
                VideoProfile::H264Baseline
                    | VideoProfile::H264Main
                    | VideoProfile::H264High
                    | VideoProfile::H265Main
                    | VideoProfile::Vp9Profile0
            )
        ) | (
            PixelFormat::Yuv420p,
            Some(
                VideoProfile::H264High10
                    | VideoProfile::H265Main10
                    | VideoProfile::Vp9Profile2
                    | VideoProfile::ProRes4444
            )
        ) | (
            PixelFormat::Yuva444p10le,
            Some(
                VideoProfile::H264Baseline
                    | VideoProfile::H264Main
                    | VideoProfile::H264High
                    | VideoProfile::H264High10
                    | VideoProfile::H265Main
                    | VideoProfile::H265Main10
                    | VideoProfile::Vp9Profile0
                    | VideoProfile::Vp9Profile2
                    | VideoProfile::Av1Main
            )
        )
    )
}

fn alpha_valid(video: &VideoOutput) -> bool {
    matches!(
        (video.alpha, video.pixel_format, video.codec),
        (
            AlphaMode::Opaque,
            PixelFormat::Yuv420p
                | PixelFormat::Yuv420p10le
                | PixelFormat::Yuv422p
                | PixelFormat::Yuv422p10le
                | PixelFormat::Yuv444p10le,
            _
        ) | (
            AlphaMode::Straight,
            PixelFormat::Yuva444p10le,
            VideoCodec::ProRes
        )
    )
}

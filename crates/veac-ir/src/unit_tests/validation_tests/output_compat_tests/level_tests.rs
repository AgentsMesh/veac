use crate::{
    video_settings_valid, AlphaMode, PixelFormat, VideoCodec, VideoProfile, VideoRateControl,
};

use super::support::video_for;

#[test]
fn codec_level_sets_accept_standard_boundaries() {
    for (codec, levels) in [
        (VideoCodec::H264, &["1", "6.2"][..]),
        (VideoCodec::H265, &["1", "6.2"][..]),
        (VideoCodec::Vp9, &["1", "6.2"][..]),
        (VideoCodec::Av1, &["2", "7.3"][..]),
    ] {
        for level in levels {
            assert!(valid(codec, level), "rejected {codec:?} level {level}");
        }
    }
}

#[test]
fn codec_level_sets_reject_other_numeric_values_and_intra_only_levels() {
    for (codec, levels) in [
        (VideoCodec::H264, &["1.5", "7", "99"][..]),
        (VideoCodec::H265, &["1.1", "6.3", "99"][..]),
        (VideoCodec::Vp9, &["1.5", "6.3", "99"][..]),
        (VideoCodec::Av1, &["1", "4.4", "8", "99"][..]),
        (VideoCodec::ProRes, &["4.1"][..]),
        (VideoCodec::DnxHr, &["4.1"][..]),
    ] {
        for level in levels {
            assert!(!valid(codec, level), "accepted {codec:?} level {level}");
        }
    }
}

fn valid(codec: VideoCodec, level: &str) -> bool {
    let mut video = video_for(codec);
    video.level = Some(level.to_owned());
    match codec {
        VideoCodec::ProRes => {
            video.pixel_format = PixelFormat::Yuva444p10le;
            video.alpha = AlphaMode::Straight;
            video.profile = Some(VideoProfile::ProRes4444);
        }
        VideoCodec::DnxHr => {
            video.pixel_format = PixelFormat::Yuv422p;
            video.profile = Some(VideoProfile::DnxHrHq);
            video.rate_control = VideoRateControl::Lossless;
        }
        _ => {}
    }
    video_settings_valid(&video)
}

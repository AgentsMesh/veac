use crate::{video_settings_valid, AlphaMode, PixelFormat, VideoCodec, VideoOutput, VideoProfile};

use super::support::video_for;

#[test]
fn every_profile_is_accepted_with_its_codec_and_bit_depth() {
    for (codec, pixel_format, profile) in profile_cases() {
        let mut video = video_for(codec);
        video.pixel_format = pixel_format;
        video.profile = Some(profile);
        if profile == VideoProfile::ProRes4444 {
            video.alpha = AlphaMode::Straight;
        }
        assert!(
            video_settings_valid(&video),
            "expected {profile:?} to be valid for {codec:?}/{pixel_format:?}"
        );
    }
}

#[test]
fn every_profile_is_rejected_by_an_unrelated_codec() {
    for (_, pixel_format, profile) in profile_cases() {
        let codec = if profile == VideoProfile::Av1Main {
            VideoCodec::H265
        } else {
            VideoCodec::Av1
        };
        let video = VideoOutput {
            codec,
            pixel_format,
            alpha: AlphaMode::Opaque,
            profile: Some(profile),
            ..video_for(codec)
        };
        assert!(
            !video_settings_valid(&video),
            "accepted {profile:?}/{codec:?}"
        );
    }
}

#[test]
fn profiles_reject_incompatible_bit_depths() {
    let ten_bit_rejects = [
        (VideoCodec::H264, VideoProfile::H264Baseline),
        (VideoCodec::H264, VideoProfile::H264Main),
        (VideoCodec::H264, VideoProfile::H264High),
        (VideoCodec::H265, VideoProfile::H265Main),
        (VideoCodec::Vp9, VideoProfile::Vp9Profile0),
    ];
    let eight_bit_rejects = [
        (VideoCodec::H264, VideoProfile::H264High10),
        (VideoCodec::H265, VideoProfile::H265Main10),
        (VideoCodec::Vp9, VideoProfile::Vp9Profile2),
        (VideoCodec::ProRes, VideoProfile::ProRes4444),
    ];

    for (codec, profile) in ten_bit_rejects {
        assert!(!settings_valid(codec, PixelFormat::Yuv420p10le, profile));
    }
    for (codec, profile) in eight_bit_rejects {
        assert!(!settings_valid(codec, PixelFormat::Yuv420p, profile));
    }
}

#[test]
fn yuva_rejects_every_non_prores_4444_profile() {
    for (codec, _, profile) in profile_cases().into_iter().take(9) {
        let video = VideoOutput {
            pixel_format: PixelFormat::Yuva444p10le,
            alpha: AlphaMode::Straight,
            profile: Some(profile),
            ..video_for(codec)
        };
        assert!(!video_settings_valid(&video), "accepted YUVA {profile:?}");
    }
}

#[test]
fn alpha_requires_prores_with_a_yuva_pixel_format() {
    for (codec, pixel_format, alpha, expected) in [
        (
            VideoCodec::H264,
            PixelFormat::Yuv420p,
            AlphaMode::Opaque,
            true,
        ),
        (
            VideoCodec::Av1,
            PixelFormat::Yuv420p10le,
            AlphaMode::Opaque,
            true,
        ),
        (
            VideoCodec::ProRes,
            PixelFormat::Yuva444p10le,
            AlphaMode::Opaque,
            false,
        ),
        (
            VideoCodec::ProRes,
            PixelFormat::Yuv420p,
            AlphaMode::Straight,
            false,
        ),
        (
            VideoCodec::ProRes,
            PixelFormat::Yuv420p10le,
            AlphaMode::Straight,
            false,
        ),
        (
            VideoCodec::Av1,
            PixelFormat::Yuva444p10le,
            AlphaMode::Straight,
            false,
        ),
        (
            VideoCodec::ProRes,
            PixelFormat::Yuva444p10le,
            AlphaMode::Straight,
            true,
        ),
    ] {
        let video = VideoOutput {
            pixel_format,
            alpha,
            ..video_for(codec)
        };
        assert_eq!(video_settings_valid(&video), expected);
    }
}

fn settings_valid(codec: VideoCodec, pixel_format: PixelFormat, profile: VideoProfile) -> bool {
    video_settings_valid(&VideoOutput {
        pixel_format,
        profile: Some(profile),
        ..video_for(codec)
    })
}

fn profile_cases() -> [(VideoCodec, PixelFormat, VideoProfile); 10] {
    [
        (
            VideoCodec::H264,
            PixelFormat::Yuv420p,
            VideoProfile::H264Baseline,
        ),
        (
            VideoCodec::H264,
            PixelFormat::Yuv420p,
            VideoProfile::H264Main,
        ),
        (
            VideoCodec::H264,
            PixelFormat::Yuv420p,
            VideoProfile::H264High,
        ),
        (
            VideoCodec::H264,
            PixelFormat::Yuv420p10le,
            VideoProfile::H264High10,
        ),
        (
            VideoCodec::H265,
            PixelFormat::Yuv420p,
            VideoProfile::H265Main,
        ),
        (
            VideoCodec::H265,
            PixelFormat::Yuv420p10le,
            VideoProfile::H265Main10,
        ),
        (
            VideoCodec::Vp9,
            PixelFormat::Yuv420p,
            VideoProfile::Vp9Profile0,
        ),
        (
            VideoCodec::Vp9,
            PixelFormat::Yuv420p10le,
            VideoProfile::Vp9Profile2,
        ),
        (VideoCodec::Av1, PixelFormat::Yuv420p, VideoProfile::Av1Main),
        (
            VideoCodec::ProRes,
            PixelFormat::Yuva444p10le,
            VideoProfile::ProRes4444,
        ),
    ]
}

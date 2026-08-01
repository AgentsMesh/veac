use crate::{
    video_settings_valid, VideoCodec, VideoOutput, VideoRateControl, MAX_VIDEO_BITRATE,
    MAX_VIDEO_BUFFER,
};

use super::support::video_for;

#[test]
fn crf_limits_are_codec_specific_and_prores_has_no_crf_mode() {
    for (codec, value, expected) in [
        (VideoCodec::H264, 51, true),
        (VideoCodec::H264, 52, false),
        (VideoCodec::H265, 51, true),
        (VideoCodec::H265, 52, false),
        (VideoCodec::Vp9, 63, true),
        (VideoCodec::Vp9, 64, false),
        (VideoCodec::Av1, 63, true),
        (VideoCodec::Av1, 64, false),
        (VideoCodec::ProRes, 0, false),
    ] {
        let video = VideoOutput {
            rate_control: VideoRateControl::Crf { value },
            ..video_for(codec)
        };
        assert_eq!(
            video_settings_valid(&video),
            expected,
            "unexpected {codec:?} CRF {value} result"
        );
    }
}

#[test]
fn bitrate_fields_enforce_positive_and_safe_integer_boundaries() {
    let valid = [
        (1, None, None),
        (2_000, Some(2_000), Some(1)),
        (
            MAX_VIDEO_BITRATE,
            Some(MAX_VIDEO_BITRATE),
            Some(MAX_VIDEO_BUFFER),
        ),
    ];
    let invalid = [
        (0, None, None),
        (MAX_VIDEO_BITRATE + 1, None, None),
        (2_000, Some(1_999), None),
        (1, Some(MAX_VIDEO_BITRATE + 1), None),
        (1, None, Some(0)),
        (1, None, Some(MAX_VIDEO_BUFFER + 1)),
    ];

    for (target_bps, max_bps, buffer_size_bits) in valid {
        assert!(valid_bitrate(target_bps, max_bps, buffer_size_bits));
    }
    for (target_bps, max_bps, buffer_size_bits) in invalid {
        assert!(!valid_bitrate(target_bps, max_bps, buffer_size_bits));
    }
}

#[test]
fn lossless_rate_control_is_accepted_for_every_codec() {
    for codec in [
        VideoCodec::H264,
        VideoCodec::H265,
        VideoCodec::Vp9,
        VideoCodec::Av1,
        VideoCodec::ProRes,
    ] {
        assert!(video_settings_valid(&VideoOutput {
            rate_control: VideoRateControl::Lossless,
            ..video_for(codec)
        }));
    }
}

#[test]
fn gop_b_frame_and_level_boundaries_are_enforced() {
    for (gop_size, b_frames, level, expected) in [
        (Some(1), Some(16), Some("4.2"), true),
        (Some(i32::MAX as u32), None, Some("5"), true),
        (Some(0), None, None, false),
        (Some(i32::MAX as u32 + 1), None, None, false),
        (None, Some(17), None, false),
        (None, None, Some(""), false),
        (None, None, Some("123456789"), false),
        (None, None, Some("5a1"), false),
        (None, None, Some("."), false),
        (None, None, Some(".1"), false),
        (None, None, Some("1."), false),
        (None, None, Some("1.2.3"), false),
        (None, None, Some("99"), false),
    ] {
        let video = VideoOutput {
            gop_size,
            b_frames,
            level: level.map(str::to_owned),
            ..VideoOutput::default()
        };
        assert_eq!(video_settings_valid(&video), expected);
    }
}

fn valid_bitrate(target_bps: u64, max_bps: Option<u64>, buffer_size_bits: Option<u64>) -> bool {
    video_settings_valid(&VideoOutput {
        rate_control: VideoRateControl::Bitrate {
            target_bps,
            max_bps,
            buffer_size_bits,
        },
        ..VideoOutput::default()
    })
}

use crate::{
    video_delivery_valid, AlphaMode, HardwareBackend, HardwareSelection, OutputFormat, PassMode,
    VideoCodec, VideoProfile, VideoRateControl,
};

use super::support::delivery_for;

#[test]
fn two_pass_requires_supported_codec_bitrate_and_software() {
    for (codec, expected) in [
        (VideoCodec::H264, true),
        (VideoCodec::H265, true),
        (VideoCodec::Vp9, false),
        (VideoCodec::Av1, false),
        (VideoCodec::ProRes, false),
    ] {
        let mut value = delivery_for(codec);
        value.pass_mode = PassMode::TwoPass;
        value.hardware = HardwareSelection::Software;
        value.video.rate_control = bitrate();
        assert_eq!(video_delivery_valid(&value), expected);
    }

    let mut value = delivery_for(VideoCodec::H264);
    value.pass_mode = PassMode::TwoPass;
    value.hardware = HardwareSelection::Software;
    assert!(!video_delivery_valid(&value));
    value.video.rate_control = VideoRateControl::Lossless;
    assert!(!video_delivery_valid(&value));
    value.video.rate_control = bitrate();
    value.hardware = HardwareSelection::Auto;
    assert!(!video_delivery_valid(&value));
}

#[test]
fn explicit_hardware_is_rejected_until_device_setup_is_modeled() {
    let codecs = [
        VideoCodec::H264,
        VideoCodec::H265,
        VideoCodec::Vp9,
        VideoCodec::Av1,
        VideoCodec::ProRes,
    ];
    for backend in [
        HardwareBackend::VideoToolbox,
        HardwareBackend::Nvenc,
        HardwareBackend::Qsv,
        HardwareBackend::Vaapi,
    ] {
        for codec in codecs {
            let mut value = delivery_for(codec);
            value.hardware = HardwareSelection::Explicit { backend };
            assert!(
                !video_delivery_valid(&value),
                "accepted {backend:?}/{codec:?}"
            );
        }
    }
}

#[test]
fn auto_and_software_hardware_modes_accept_every_codec() {
    for hardware in [HardwareSelection::Auto, HardwareSelection::Software] {
        for codec in [
            VideoCodec::H264,
            VideoCodec::H265,
            VideoCodec::Vp9,
            VideoCodec::Av1,
            VideoCodec::ProRes,
        ] {
            let mut value = delivery_for(codec);
            value.hardware = hardware;
            assert!(video_delivery_valid(&value));
        }
    }
}

#[test]
fn x265_and_libaom_explicit_levels_fail_closed() {
    for codec in [VideoCodec::H265, VideoCodec::Av1] {
        let mut value = delivery_for(codec);
        value.video.level = Some("5.1".to_owned());
        assert!(!video_delivery_valid(&value));
    }
    for codec in [VideoCodec::H264, VideoCodec::Vp9] {
        let mut value = delivery_for(codec);
        value.video.level = Some("5.1".to_owned());
        assert!(video_delivery_valid(&value));
    }
}

#[test]
fn prores_delivery_requires_lossless_rate_control() {
    let mut value = delivery_for(VideoCodec::ProRes);
    assert!(video_delivery_valid(&value));
    value.video.rate_control = VideoRateControl::Crf { value: 0 };
    assert!(!video_delivery_valid(&value));
    value.video.rate_control = bitrate();
    assert!(!video_delivery_valid(&value));
}

#[test]
fn straight_alpha_delivery_requires_mov_and_prores_4444_profile() {
    let mut value = delivery_for(VideoCodec::ProRes);
    value.video.alpha = AlphaMode::Straight;
    value.video.profile = Some(VideoProfile::ProRes4444);

    for (container, expected) in [
        (OutputFormat::Mov, true),
        (OutputFormat::Mp4, false),
        (OutputFormat::Mkv, false),
        (OutputFormat::Webm, false),
    ] {
        value.container = container;
        assert_eq!(video_delivery_valid(&value), expected);
    }

    value.container = OutputFormat::Mov;
    value.video.profile = None;
    assert!(!video_delivery_valid(&value));
    value.video.alpha = AlphaMode::Opaque;
    assert!(!video_delivery_valid(&value));
}

fn bitrate() -> VideoRateControl {
    VideoRateControl::Bitrate {
        target_bps: 1_000_000,
        max_bps: None,
        buffer_size_bits: None,
    }
}

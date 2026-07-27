use veac_ir::{PixelFormat, VideoCodec, VideoProfile};

use super::{codec, level_matches, parse_level, pixel_format_matches, profile_matches};

#[test]
fn every_authored_video_codec_has_an_exact_probe_name() {
    let cases = [
        (VideoCodec::H264, "h264"),
        (VideoCodec::H265, "hevc"),
        (VideoCodec::Vp9, "vp9"),
        (VideoCodec::Av1, "av1"),
        (VideoCodec::ProRes, "prores"),
        (VideoCodec::DnxHr, "dnxhd"),
    ];
    for (authored, observed) in cases {
        assert_eq!(codec(authored), observed);
    }
}

#[test]
fn every_authored_video_profile_matches_only_its_probe_family() {
    let cases = [
        (VideoProfile::H264Baseline, "Constrained Baseline"),
        (VideoProfile::H264Main, "Main"),
        (VideoProfile::H264High, "High"),
        (VideoProfile::H264High10, "High 10 Intra"),
        (VideoProfile::H265Main, "Main"),
        (VideoProfile::H265Main10, "Main 10 Intra"),
        (VideoProfile::Vp9Profile0, "Profile 0"),
        (VideoProfile::Vp9Profile2, "Profile 2"),
        (VideoProfile::Av1Main, "Main"),
        (VideoProfile::ProRes4444, "4444"),
        (VideoProfile::DnxHrLb, "DNXHR LB"),
        (VideoProfile::DnxHrSq, "DNXHR SQ"),
        (VideoProfile::DnxHrHq, "DNXHR HQ"),
        (VideoProfile::DnxHrHqx, "DNXHR HQX"),
        (VideoProfile::DnxHr444, "DNXHR 444"),
    ];
    for (authored, observed) in cases {
        assert!(profile_matches(Some(authored), Some(observed)));
        assert!(!profile_matches(Some(authored), Some("unrelated")));
    }
    assert!(profile_matches(None, None));
    assert!(!profile_matches(Some(VideoProfile::H264Main), None));
}

#[test]
fn levels_and_controlled_prores_pixel_expansion_are_exact() {
    assert_eq!(parse_level("4"), Some(40));
    assert_eq!(parse_level("4.1"), Some(41));
    assert_eq!(parse_level("4.10"), None);
    assert_eq!(parse_level("4.1.1"), None);
    assert_eq!(parse_level("invalid"), None);
    assert!(level_matches(None, None));
    assert!(level_matches(Some("4.1"), Some(41)));
    assert!(!level_matches(Some("4.1"), Some(42)));
    assert!(pixel_format_matches(
        VideoCodec::ProRes,
        Some(VideoProfile::ProRes4444),
        PixelFormat::Yuva444p10le,
        "yuva444p12le",
    ));
    assert!(!pixel_format_matches(
        VideoCodec::H264,
        None,
        PixelFormat::Yuv420p,
        "yuv444p",
    ));
}

use veac_ir::{PixelFormat, VideoCodec, VideoProfile};

use super::{
    codec, level_matches, parse_level, pixel_format, pixel_format_matches, profile_matches,
};

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
fn every_authored_pixel_format_has_an_exact_probe_name() {
    for (authored, observed) in [
        (PixelFormat::Yuv420p, "yuv420p"),
        (PixelFormat::Yuv420p10le, "yuv420p10le"),
        (PixelFormat::Yuv422p, "yuv422p"),
        (PixelFormat::Yuv422p10le, "yuv422p10le"),
        (PixelFormat::Yuv444p10le, "yuv444p10le"),
        (PixelFormat::Yuva444p10le, "yuva444p10le"),
    ] {
        assert_eq!(pixel_format(authored), observed);
    }
}

#[test]
fn every_authored_video_profile_matches_only_its_probe_family() {
    let cases: &[(VideoProfile, &[&str])] = &[
        (
            VideoProfile::H264Baseline,
            &["Baseline", "Constrained Baseline", "66", "578"],
        ),
        (VideoProfile::H264Main, &["Main", "77"]),
        (VideoProfile::H264High, &["High", "100"]),
        (
            VideoProfile::H264High10,
            &["High 10", "High 10 Intra", "110", "2158"],
        ),
        (VideoProfile::H265Main, &["Main", "1"]),
        (VideoProfile::H265Main10, &["Main 10", "Main 10 Intra", "2"]),
        (VideoProfile::Vp9Profile0, &["Profile 0", "0"]),
        (VideoProfile::Vp9Profile2, &["Profile 2", "2"]),
        (VideoProfile::Av1Main, &["Main", "0"]),
        (VideoProfile::ProRes4444, &["4444", "4"]),
        (VideoProfile::DnxHrLb, &["DNXHR LB", "1"]),
        (VideoProfile::DnxHrSq, &["DNXHR SQ", "2"]),
        (VideoProfile::DnxHrHq, &["DNXHR HQ", "3"]),
        (VideoProfile::DnxHrHqx, &["DNXHR HQX", "4"]),
        (VideoProfile::DnxHr444, &["DNXHR 444", "5"]),
    ];
    for (authored, accepted) in cases {
        for observed in *accepted {
            assert!(profile_matches(Some(*authored), Some(observed)));
        }
        assert!(!profile_matches(Some(*authored), Some("unrelated")));
    }
    for (authored, sibling) in [
        (VideoProfile::H264Baseline, "77"),
        (VideoProfile::H264Main, "100"),
        (VideoProfile::H264High, "110"),
        (VideoProfile::H264High10, "66"),
        (VideoProfile::H265Main, "2"),
        (VideoProfile::Vp9Profile0, "2"),
        (VideoProfile::DnxHrHqx, "5"),
    ] {
        assert!(!profile_matches(Some(authored), Some(sibling)));
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

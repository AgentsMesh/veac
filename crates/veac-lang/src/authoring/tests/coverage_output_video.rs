use super::coverage_output_support::video;

#[test]
fn video_container_codec_and_pixel_format_variants_lower_exactly() {
    use veac_ir::{OutputFormat as F, PixelFormat as P, VideoCodec as C};
    for (token, settings, expected) in [
        ("mp4", "", F::Mp4),
        ("mov", "", F::Mov),
        ("mkv", "", F::Mkv),
        ("webm", "codec vp9;", F::Webm),
        (
            "mxf",
            "codec dnxhr; pixel-format yuv422p; profile dnxhr-hq; rate-control lossless;",
            F::Mxf,
        ),
    ] {
        assert_eq!(
            video(settings, &format!("container {token}; audio none;")).container,
            expected
        );
    }
    for (settings, encoding, expected) in [
        ("codec h264;", "audio none;", C::H264),
        ("codec h265;", "audio none;", C::H265),
        ("codec vp9;", "container webm; audio none;", C::Vp9),
        ("codec av1;", "audio none;", C::Av1),
        (
            "codec prores; pixel-format yuva444p10le; alpha straight; profile prores-4444; rate-control lossless;",
            "container mov; audio none;",
            C::ProRes,
        ),
        (
            "codec dnxhr; pixel-format yuv422p; profile dnxhr-hq; rate-control lossless;",
            "container mxf; audio none;",
            C::DnxHr,
        ),
    ] {
        assert_eq!(video(settings, encoding).video.codec, expected);
    }
    for (settings, encoding, expected) in [
        ("pixel-format yuv420p;", "audio none;", P::Yuv420p),
        ("pixel-format yuv420p10le;", "audio none;", P::Yuv420p10le),
        (
            "codec dnxhr; pixel-format yuv422p; profile dnxhr-hq; rate-control lossless;",
            "container mxf; audio none;",
            P::Yuv422p,
        ),
        (
            "codec dnxhr; pixel-format yuv422p10le; profile dnxhr-hqx; rate-control lossless;",
            "container mxf; audio none;",
            P::Yuv422p10le,
        ),
        (
            "codec dnxhr; pixel-format yuv444p10le; profile dnxhr-444; rate-control lossless;",
            "container mxf; audio none;",
            P::Yuv444p10le,
        ),
        (
            "codec prores; pixel-format yuva444p10le; alpha straight; profile prores-4444; rate-control lossless;",
            "container mov; audio none;",
            P::Yuva444p10le,
        ),
    ] {
        assert_eq!(video(settings, encoding).video.pixel_format, expected);
    }
}

#[test]
fn every_video_profile_lowers_exactly() {
    use veac_ir::VideoProfile as P;
    for (settings, encoding, expected) in [
        ("codec h264; profile h264-baseline;", "audio none;", P::H264Baseline),
        ("codec h264; profile h264-main;", "audio none;", P::H264Main),
        ("codec h264; profile h264-high;", "audio none;", P::H264High),
        ("codec h264; pixel-format yuv420p10le; profile h264-high10;", "audio none;", P::H264High10),
        ("codec h265; profile h265-main;", "audio none;", P::H265Main),
        ("codec h265; pixel-format yuv420p10le; profile h265-main10;", "audio none;", P::H265Main10),
        ("codec vp9; profile vp9-profile0;", "container webm; audio none;", P::Vp9Profile0),
        ("codec vp9; pixel-format yuv420p10le; profile vp9-profile2;", "container webm; audio none;", P::Vp9Profile2),
        ("codec av1; profile av1-main;", "audio none;", P::Av1Main),
        ("codec prores; pixel-format yuva444p10le; alpha straight; profile prores-4444; rate-control lossless;", "container mov; audio none;", P::ProRes4444),
        ("codec dnxhr; pixel-format yuv422p; profile dnxhr-lb; rate-control lossless;", "container mxf; audio none;", P::DnxHrLb),
        ("codec dnxhr; pixel-format yuv422p; profile dnxhr-sq; rate-control lossless;", "container mxf; audio none;", P::DnxHrSq),
        ("codec dnxhr; pixel-format yuv422p; profile dnxhr-hq; rate-control lossless;", "container mxf; audio none;", P::DnxHrHq),
        ("codec dnxhr; pixel-format yuv422p10le; profile dnxhr-hqx; rate-control lossless;", "container mxf; audio none;", P::DnxHrHqx),
        ("codec dnxhr; pixel-format yuv444p10le; profile dnxhr-444; rate-control lossless;", "container mxf; audio none;", P::DnxHr444),
    ] {
        assert_eq!(video(settings, encoding).video.profile, Some(expected));
    }
}

#[test]
fn rate_control_and_optional_video_fields_preserve_values() {
    let crf = video(
        "rate-control crf { value 31; } gop-size 48; b-frames 3; level \"5.1\";",
        "audio none; optimize-for-streaming true;",
    );
    assert_eq!(
        crf.video.rate_control,
        veac_ir::VideoRateControl::Crf { value: 31 }
    );
    assert_eq!(
        (crf.video.gop_size, crf.video.b_frames),
        (Some(48), Some(3))
    );
    assert_eq!(crf.video.level.as_deref(), Some("5.1"));
    assert!(crf.optimize_for_streaming);

    let bitrate = video(
        "rate-control bitrate { target-bps 1; max-bps 2; buffer-bps 3; }",
        "audio none;",
    );
    assert_eq!(
        bitrate.video.rate_control,
        veac_ir::VideoRateControl::Bitrate {
            target_bps: 1,
            max_bps: Some(2),
            buffer_size_bits: Some(3),
        }
    );
    assert_eq!(
        video("rate-control lossless;", "audio none;")
            .video
            .rate_control,
        veac_ir::VideoRateControl::Lossless
    );
    let two_pass = video(
        "rate-control average { target 100bps; }",
        "audio none; pass-mode two-pass; hardware software;",
    );
    assert_eq!(two_pass.pass_mode, veac_ir::PassMode::TwoPass);
    assert_eq!(two_pass.hardware, veac_ir::HardwareSelection::Software);
    let alpha = video(
        "codec prores; pixel-format yuva444p10le; alpha straight; profile prores-4444; rate-control lossless;",
        "container mov; audio none;",
    );
    assert_eq!(alpha.video.alpha, veac_ir::AlphaMode::Straight);
}

#[test]
fn hardware_and_audio_variants_lower_exactly() {
    use veac_ir::{AudioCodec as A, HardwareSelection as H};
    assert_eq!(video("", "audio none; hardware auto;").hardware, H::Auto);
    for token in ["videotoolbox", "nvenc", "qsv", "vaapi"] {
        let error = super::coverage_output_support::video_result(
            "",
            &format!("audio none; hardware {token};"),
        )
        .unwrap_err();
        assert!(error.as_slice().iter().any(|value| {
            value.code == "AUTHORING_LOWER_IR_VALIDATION"
                && value.message.contains("OUTPUT_VIDEO_SETTINGS")
        }));
    }
    for (token, encoding, codec) in [
        ("aac", "", A::Aac),
        ("opus", "container webm;", A::Opus),
        ("flac", "container mkv;", A::Flac),
        ("pcm-s16le", "container mov;", A::PcmS16Le),
        ("pcm-s24le", "container mov;", A::PcmS24Le),
        ("pcm-s32le", "container mov;", A::PcmS32Le),
    ] {
        let video_settings = if token == "opus" { "codec vp9;" } else { "" };
        let audio = video(
            video_settings,
            &format!("{encoding} audio {{ codec {token}; sample-rate 48000; channels 6; }}"),
        )
        .audio
        .unwrap();
        assert_eq!(
            (audio.codec, audio.sample_rate, audio.channels),
            (codec, 48_000, 6)
        );
    }
}

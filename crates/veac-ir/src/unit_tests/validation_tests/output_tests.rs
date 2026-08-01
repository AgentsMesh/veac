use super::*;

#[test]
fn output_compatibility_matrix_is_explicit() {
    assert!(video_container_compatible(
        OutputFormat::Mp4,
        VideoCodec::H264
    ));
    assert!(video_container_compatible(
        OutputFormat::Mov,
        VideoCodec::H265
    ));
    assert!(video_container_compatible(
        OutputFormat::Mkv,
        VideoCodec::Vp9
    ));
    assert!(video_container_compatible(
        OutputFormat::Webm,
        VideoCodec::Av1
    ));
    assert!(!video_container_compatible(
        OutputFormat::Mp4,
        VideoCodec::Vp9
    ));
    assert!(!video_container_compatible(
        OutputFormat::Mov,
        VideoCodec::Av1
    ));
    assert!(!video_container_compatible(
        OutputFormat::Webm,
        VideoCodec::H264
    ));

    assert!(audio_container_compatible(
        OutputFormat::Mp4,
        AudioCodec::Aac
    ));
    assert!(audio_container_compatible(
        OutputFormat::Mov,
        AudioCodec::PcmS16Le
    ));
    assert!(audio_container_compatible(
        OutputFormat::Mkv,
        AudioCodec::Flac
    ));
    assert!(audio_container_compatible(
        OutputFormat::Webm,
        AudioCodec::Opus
    ));
    assert!(!audio_container_compatible(
        OutputFormat::Mp4,
        AudioCodec::Opus
    ));
    assert!(!audio_container_compatible(
        OutputFormat::Mov,
        AudioCodec::Flac
    ));
    assert!(!audio_container_compatible(
        OutputFormat::Webm,
        AudioCodec::Aac
    ));

    assert!(output_file_compatible("DELIVERY.MP4", OutputFormat::Mp4));
    assert!(output_file_compatible("delivery.mov", OutputFormat::Mov));
    assert!(output_file_compatible("delivery.mkv", OutputFormat::Mkv));
    assert!(output_file_compatible("delivery.webm", OutputFormat::Webm));
    assert!(!output_file_compatible("delivery", OutputFormat::Mp4));
    assert!(!output_file_compatible("delivery.mov", OutputFormat::Mp4));
}

#[test]
fn video_setting_validation_covers_rate_profile_pixel_and_level_rules() {
    let valid = VideoOutput::default();
    assert!(video_settings_valid(&valid));
    for rate_control in [
        VideoRateControl::Crf { value: 52 },
        VideoRateControl::Bitrate {
            target_bps: 0,
            max_bps: None,
            buffer_size_bits: None,
        },
        VideoRateControl::Bitrate {
            target_bps: 2_000,
            max_bps: Some(1_000),
            buffer_size_bits: Some(0),
        },
    ] {
        assert!(!video_settings_valid(&VideoOutput {
            rate_control,
            ..valid.clone()
        }));
    }
    for invalid in [
        VideoOutput {
            gop_size: Some(0),
            ..valid.clone()
        },
        VideoOutput {
            b_frames: Some(17),
            ..valid.clone()
        },
        VideoOutput {
            profile: Some(VideoProfile::Vp9Profile0),
            ..valid.clone()
        },
        VideoOutput {
            pixel_format: PixelFormat::Yuv420p10le,
            profile: Some(VideoProfile::H264High),
            ..valid.clone()
        },
        VideoOutput {
            profile: Some(VideoProfile::H264High10),
            ..valid.clone()
        },
        VideoOutput {
            level: Some("bad-level".to_owned()),
            ..valid.clone()
        },
    ] {
        assert!(!video_settings_valid(&invalid));
    }
    let ten_bit = VideoOutput {
        codec: VideoCodec::H265,
        pixel_format: PixelFormat::Yuv420p10le,
        profile: Some(VideoProfile::H265Main10),
        rate_control: VideoRateControl::Crf { value: 28 },
        level: Some("5.1".to_owned()),
        ..valid
    };
    assert!(video_settings_valid(&ten_bit));
    for (codec, pixel_format, profile, value) in [
        (
            VideoCodec::H264,
            PixelFormat::Yuv420p,
            VideoProfile::H264Baseline,
            51,
        ),
        (
            VideoCodec::Vp9,
            PixelFormat::Yuv420p,
            VideoProfile::Vp9Profile0,
            63,
        ),
        (
            VideoCodec::Av1,
            PixelFormat::Yuv420p,
            VideoProfile::Av1Main,
            63,
        ),
    ] {
        assert!(video_settings_valid(&VideoOutput {
            codec,
            pixel_format,
            profile: Some(profile),
            rate_control: VideoRateControl::Crf { value },
            ..VideoOutput::default()
        }));
    }
    assert!(video_settings_valid(&VideoOutput {
        rate_control: VideoRateControl::Bitrate {
            target_bps: 2_000,
            max_bps: Some(3_000),
            buffer_size_bits: Some(4_000),
        },
        ..VideoOutput::default()
    }));
}

#[test]
fn render_config_reports_all_export_contract_failures() {
    let mut project = sample_project();
    let output = &mut project.project.render_configs[0];
    let deliverable = &mut output.deliverables[0];
    deliverable.target = DeliverableTarget::File {
        name: "delivery.mp4".to_owned(),
    };
    let DeliverableKind::Video(settings) = &mut deliverable.kind else {
        panic!("sample project must contain a video deliverable")
    };
    settings.container = OutputFormat::Webm;
    settings.video.codec = VideoCodec::H264;
    settings.video.rate_control = VideoRateControl::Crf { value: 99 };
    settings.audio.as_mut().unwrap().codec = AudioCodec::Aac;
    settings.optimize_for_streaming = true;
    let codes = validation_codes(&project);
    for expected in [
        "OUTPUT_VIDEO_SETTINGS",
        "OUTPUT_VIDEO_CODEC",
        "OUTPUT_AUDIO",
        "OUTPUT_FILE_FORMAT",
        "OUTPUT_STREAMING_MODE",
    ] {
        assert_code(&codes, expected);
    }
}

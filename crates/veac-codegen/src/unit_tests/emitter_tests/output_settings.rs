use veac_plan::canonical::*;

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn emits_quality_gop_profile_level_pixel_format_and_fast_start() {
    let mut plan = resolved(&fixture());
    let delivery = delivery(&mut plan);
    delivery.video = VideoOutput {
        codec: VideoCodec::H264,
        pixel_format: PixelFormat::Yuv420p,
        alpha: AlphaMode::Opaque,
        color_space: Some(ColorSpace {
            primaries: ColorPrimaries::Bt709,
            transfer: ColorTransfer::Bt709,
            matrix: ColorMatrix::Bt709,
            range: ColorRange::Limited,
        }),
        rate_control: VideoRateControl::Crf { value: 18 },
        gop_size: Some(60),
        b_frames: Some(3),
        profile: Some(VideoProfile::H264High),
        level: Some("4.1".to_owned()),
    };
    delivery.optimize_for_streaming = true;
    let args = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .output_args;
    for (name, value) in [
        ("-pix_fmt", "yuv420p"),
        ("-crf", "18"),
        ("-g", "60"),
        ("-bf", "3"),
        ("-profile:v", "high"),
        ("-level:v", "4.1"),
        ("-color_primaries", "bt709"),
        ("-color_trc", "bt709"),
        ("-colorspace", "bt709"),
        ("-color_range", "tv"),
        ("-movflags", "+faststart"),
    ] {
        assert!(pair(&args, name, value), "missing {name} {value}: {args:?}");
    }
}

#[test]
fn emits_bounded_bitrate_and_codec_specific_lossless_modes() {
    let mut plan = resolved(&fixture());
    delivery(&mut plan).video.rate_control = VideoRateControl::Bitrate {
        target_bps: 4_000_000,
        max_bps: Some(6_000_000),
        buffer_size_bits: Some(8_000_000),
    };
    let args = emit_video_command(&plan, &bindings(&plan))
        .unwrap()
        .output_args;
    assert!(pair(&args, "-b:v", "4000000"));
    assert!(pair(&args, "-maxrate", "6000000"));
    assert!(pair(&args, "-bufsize", "8000000"));

    for (codec, expected_name, expected_value) in [
        (VideoCodec::H264, "-crf", "0"),
        (VideoCodec::H265, "-crf", "0"),
        (VideoCodec::Vp9, "-lossless", "1"),
        (VideoCodec::Av1, "-crf", "0"),
    ] {
        let mut plan = resolved(&fixture());
        plan.output.deliverables[0].target = DeliverableTarget::File {
            name: "output.mkv".to_owned(),
        };
        let delivery = delivery(&mut plan);
        delivery.container = OutputFormat::Mkv;
        delivery.video.codec = codec;
        delivery.video.rate_control = VideoRateControl::Lossless;
        let args = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .output_args;
        assert!(pair(&args, expected_name, expected_value));
        if matches!(codec, VideoCodec::Vp9 | VideoCodec::Av1) {
            assert!(pair(&args, "-b:v", "0"));
        }
    }
}

#[test]
fn emits_every_codec_specific_profile_and_ten_bit_pixel_format() {
    let cases = [
        (VideoCodec::H264, VideoProfile::H264Baseline, "baseline"),
        (VideoCodec::H264, VideoProfile::H264Main, "main"),
        (VideoCodec::H264, VideoProfile::H264High10, "high10"),
        (VideoCodec::H265, VideoProfile::H265Main, "main"),
        (VideoCodec::H265, VideoProfile::H265Main10, "main10"),
        (VideoCodec::Vp9, VideoProfile::Vp9Profile0, "0"),
        (VideoCodec::Vp9, VideoProfile::Vp9Profile2, "2"),
        (VideoCodec::Av1, VideoProfile::Av1Main, "0"),
    ];
    for (codec, profile, encoded) in cases {
        let mut plan = resolved(&fixture());
        plan.output.deliverables[0].target = DeliverableTarget::File {
            name: "output.mkv".to_owned(),
        };
        let delivery = delivery(&mut plan);
        delivery.container = OutputFormat::Mkv;
        delivery.video.codec = codec;
        delivery.video.profile = Some(profile);
        delivery.video.pixel_format = if matches!(
            profile,
            VideoProfile::H264High10 | VideoProfile::H265Main10 | VideoProfile::Vp9Profile2
        ) {
            PixelFormat::Yuv420p10le
        } else {
            PixelFormat::Yuv420p
        };
        let ten_bit = delivery.video.pixel_format == PixelFormat::Yuv420p10le;
        let args = emit_video_command(&plan, &bindings(&plan))
            .unwrap()
            .output_args;
        assert!(pair(&args, "-profile:v", encoded));
        assert!(pair(
            &args,
            "-pix_fmt",
            if ten_bit { "yuv420p10le" } else { "yuv420p" }
        ));
    }
}

#[test]
fn preflight_rejects_invalid_encoder_and_streaming_settings() {
    let mut plan = resolved(&fixture());
    delivery(&mut plan).video.rate_control = VideoRateControl::Crf { value: 99 };
    assert!(codes(&plan).contains(&"PLAN_VIDEO_SETTINGS_INVALID"));

    let mut plan = resolved(&fixture());
    let delivery = delivery(&mut plan);
    delivery.container = OutputFormat::Webm;
    delivery.video.codec = VideoCodec::Vp9;
    delivery.optimize_for_streaming = true;
    assert!(codes(&plan).contains(&"PLAN_STREAMING_MODE_INVALID"));
}

fn delivery(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut VideoDeliverable {
    plan.output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
}

fn codes(plan: &veac_plan::ResolvedRenderPlan) -> Vec<&'static str> {
    emit_video_command(plan, &bindings(plan))
        .unwrap_err()
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

fn pair(args: &[String], name: &str, value: &str) -> bool {
    args.windows(2).any(|pair| pair == [name, value])
}

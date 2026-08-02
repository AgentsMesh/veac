use veac_plan::canonical::*;

use super::support::{bindings, emit_video_command, fixture, resolved};

#[test]
fn render_geometry_must_fit_ffmpeg_option_integers_and_resource_budgets() {
    let mutations: [fn(&mut veac_plan::ResolvedRenderPlan); 7] = [
        |plan: &mut veac_plan::ResolvedRenderPlan| raster(plan).width = i32::MAX as u32 + 1,
        |plan: &mut veac_plan::ResolvedRenderPlan| raster(plan).height = i32::MAX as u32 + 1,
        |plan: &mut veac_plan::ResolvedRenderPlan| {
            raster(plan).frame_rate.numerator = i64::from(i32::MAX) + 1;
        },
        |plan: &mut veac_plan::ResolvedRenderPlan| {
            raster(plan).frame_rate.denominator = i32::MAX as u32 + 1;
        },
        |plan: &mut veac_plan::ResolvedRenderPlan| raster(plan).width = MAX_DIMENSION + 1,
        |plan: &mut veac_plan::ResolvedRenderPlan| {
            let raster = raster(plan);
            (raster.width, raster.height) = (7_680, 4_320);
        },
        |plan: &mut veac_plan::ResolvedRenderPlan| {
            raster(plan).frame_rate = Rational::new(i64::from(MAX_FRAME_RATE) + 1, 1).unwrap();
        },
    ];
    for mutate in mutations {
        let mut plan = resolved(&fixture());
        mutate(&mut plan);
        assert_code(&plan, "PLAN_RASTER_INVALID");
    }
}

#[test]
fn sequence_and_scope_geometry_must_fit_ffmpeg_option_integers() {
    let mut sequence = resolved(&fixture());
    (
        sequence.sequences[0].settings.width,
        sequence.sequences[0].settings.height,
    ) = (7_680, 4_320);
    assert_code(&sequence, "PLAN_STRUCTURE_INVALID");

    let mut scope = resolved(&fixture());
    scope.output.deliverables[0].target = DeliverableTarget::File {
        name: "scope.png".to_owned(),
    };
    scope.output.deliverables[0].kind = DeliverableKind::Scope(ScopeOutput {
        scope: VideoScope::Histogram,
        at: RationalTime::zero(600).unwrap(),
        width: 7_680,
        height: 4_320,
        format: ImageFormat::Png,
    });
    assert_code(&scope, "PLAN_SCOPE_OUTPUT_INVALID");
}

#[test]
fn video_settings_reject_unaligned_pixels_gop_overflow_and_malformed_levels() {
    let mut odd = resolved(&fixture());
    raster(&mut odd).width |= 1;
    assert_code(&odd, "PLAN_VIDEO_SETTINGS_INVALID");

    let mut gop = resolved(&fixture());
    delivery(&mut gop).video.gop_size = Some(i32::MAX as u32 + 1);
    assert_code(&gop, "PLAN_VIDEO_SETTINGS_INVALID");

    for level in [".", ".1", "1.", "1.2.3", "99"] {
        let mut plan = resolved(&fixture());
        delivery(&mut plan).video.level = Some(level.to_owned());
        assert_code(&plan, "PLAN_VIDEO_SETTINGS_INVALID");
    }
    for codec in [VideoCodec::H265, VideoCodec::Av1] {
        let mut plan = resolved(&fixture());
        delivery(&mut plan).video.codec = codec;
        delivery(&mut plan).video.level = Some("5.1".to_owned());
        assert_code(&plan, "PLAN_VIDEO_SETTINGS_INVALID");
    }
}

#[test]
fn prores_rejects_a_pixel_format_the_encoder_would_silently_replace() {
    let mut plan = resolved(&fixture());
    plan.output.deliverables[0].target = DeliverableTarget::File {
        name: "master.mov".to_owned(),
    };
    let delivery = delivery(&mut plan);
    delivery.container = OutputFormat::Mov;
    delivery.video.codec = VideoCodec::ProRes;
    delivery.video.rate_control = VideoRateControl::Lossless;
    assert_code(&plan, "PLAN_VIDEO_SETTINGS_INVALID");
}

#[test]
fn audio_output_validation_is_codec_aware_and_limited_to_eight_channels() {
    for audio in [
        AudioOutput {
            codec: AudioCodec::Aac,
            sample_rate: 1,
            channels: 2,
        },
        AudioOutput {
            codec: AudioCodec::Opus,
            sample_rate: 44_100,
            channels: 2,
        },
        AudioOutput {
            codec: AudioCodec::Flac,
            sample_rate: 655_351,
            channels: 2,
        },
        AudioOutput {
            codec: AudioCodec::PcmS16Le,
            sample_rate: 384_001,
            channels: 2,
        },
        AudioOutput {
            codec: AudioCodec::Aac,
            sample_rate: 48_000,
            channels: 9,
        },
    ] {
        let mut plan = resolved(&fixture());
        delivery(&mut plan).audio = Some(audio);
        assert_code(&plan, "PLAN_AUDIO_OUTPUT_INVALID");
    }
}

#[test]
fn rate_control_rejects_untrusted_plan_resource_overflow() {
    let mut plan = resolved(&fixture());
    delivery(&mut plan).video.rate_control = VideoRateControl::Bitrate {
        target_bps: MAX_VIDEO_BITRATE + 1,
        max_bps: None,
        buffer_size_bits: Some(MAX_VIDEO_BUFFER + 1),
    };
    assert_code(&plan, "PLAN_VIDEO_SETTINGS_INVALID");
}

#[test]
fn image_sequence_last_number_must_fit_ffmpeg_integer() {
    let mut plan = resolved(&fixture());
    plan.output.deliverables[0].target = DeliverableTarget::ImageSequence {
        pattern: "frame-%d.png".to_owned(),
    };
    plan.output.deliverables[0].kind = DeliverableKind::ImageSequence(ImageSequenceOutput {
        format: ImageFormat::Png,
        start_number: i32::MAX as u32,
    });
    assert_code(&plan, "PLAN_IMAGE_SEQUENCE_INVALID");
}

#[test]
fn audio_stem_reuses_the_codec_aware_output_contract() {
    let mut plan = resolved(&fixture());
    plan.output.deliverables[0].target = DeliverableTarget::File {
        name: "master.wav".to_owned(),
    };
    plan.output.deliverables[0].kind = DeliverableKind::AudioStem(AudioStemOutput {
        format: AudioStemFormat::Wav,
        audio: AudioOutput {
            codec: AudioCodec::PcmS16Le,
            sample_rate: 384_001,
            channels: 2,
        },
        source: AudioMixSource::Master,
    });
    assert_code(&plan, "PLAN_AUDIO_STEM_INVALID");
}

fn delivery(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut VideoDeliverable {
    plan.output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
}

fn raster(plan: &mut veac_plan::ResolvedRenderPlan) -> &mut RasterSettings {
    plan.output.raster.as_mut().unwrap()
}

fn assert_code(plan: &veac_plan::ResolvedRenderPlan, expected: &str) {
    let error = emit_video_command(plan, &bindings(plan)).unwrap_err();
    assert!(
        error
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.code == expected),
        "missing {expected}: {error}"
    );
}

use veac_codegen::emitter::{emit_all, BackendAction, BackendRequirement};
use veac_plan::canonical::*;

use super::support::{bindings, fixture, resolved};

#[test]
fn every_dnxhr_profile_emits_its_exact_ffmpeg_contract() {
    for (profile, pixel, encoded) in [
        (VideoProfile::DnxHrLb, PixelFormat::Yuv422p, "dnxhr_lb"),
        (VideoProfile::DnxHrSq, PixelFormat::Yuv422p, "dnxhr_sq"),
        (VideoProfile::DnxHrHq, PixelFormat::Yuv422p, "dnxhr_hq"),
        (
            VideoProfile::DnxHrHqx,
            PixelFormat::Yuv422p10le,
            "dnxhr_hqx",
        ),
        (
            VideoProfile::DnxHr444,
            PixelFormat::Yuv444p10le,
            "dnxhr_444",
        ),
    ] {
        let plan = plan(profile, pixel);
        let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
        let args = command(&bundle.tasks()[0]).output_args.as_slice();
        for (name, value) in [
            ("-c:v", "dnxhd"),
            ("-profile:v", encoded),
            ("-pix_fmt", pixel_name(pixel)),
            ("-f", "mxf"),
            ("-c:a", "pcm_s24le"),
            ("-ar", "48000"),
        ] {
            assert!(pair(args, name, value), "missing {name} {value}: {args:?}");
        }
        assert!(!args.iter().any(|value| value == "-crf"));
        assert!(requirement(bundle.requirements(), "encoder", "dnxhd"));
        assert!(requirement(bundle.requirements(), "encoder", "pcm_s24le"));
        assert!(requirement(bundle.requirements(), "muxer", "mxf"));
    }
}

#[test]
fn dnxhr_crossed_pixels_and_mxf_geometry_fail_before_emission() {
    for (profile, pixel) in [
        (VideoProfile::DnxHrHq, PixelFormat::Yuv422p10le),
        (VideoProfile::DnxHrHqx, PixelFormat::Yuv422p),
        (VideoProfile::DnxHr444, PixelFormat::Yuv422p10le),
    ] {
        assert!(codes(&plan(profile, pixel)).contains(&"PLAN_VIDEO_SETTINGS_INVALID"));
    }
    let mut too_small = plan(VideoProfile::DnxHrHq, PixelFormat::Yuv422p);
    let raster = too_small.output.raster.as_mut().unwrap();
    (raster.width, raster.height) = (96, 54);
    assert!(codes(&too_small).contains(&"PLAN_MXF_OUTPUT_INVALID"));
    let mut bad_rate = plan(VideoProfile::DnxHrHq, PixelFormat::Yuv422p);
    bad_rate.output.raster.as_mut().unwrap().frame_rate = Rational::new(10, 1).unwrap();
    assert!(codes(&bad_rate).contains(&"PLAN_MXF_OUTPUT_INVALID"));
}

fn plan(profile: VideoProfile, pixel: PixelFormat) -> veac_plan::ResolvedRenderPlan {
    let mut plan = resolved(&fixture());
    let raster = plan.output.raster.as_mut().unwrap();
    (raster.width, raster.height) = (256, 120);
    raster.frame_rate = Rational::new(24, 1).unwrap();
    plan.output.deliverables[0].target = DeliverableTarget::File {
        name: "master.mxf".to_owned(),
    };
    let delivery = plan
        .output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap();
    delivery.container = OutputFormat::Mxf;
    delivery.video = VideoOutput {
        codec: VideoCodec::DnxHr,
        pixel_format: pixel,
        alpha: AlphaMode::Opaque,
        color_space: None,
        rate_control: VideoRateControl::Lossless,
        gop_size: None,
        b_frames: None,
        profile: Some(profile),
        level: None,
    };
    delivery.audio = Some(AudioOutput {
        codec: AudioCodec::PcmS24Le,
        sample_rate: 48_000,
        channels: 2,
    });
    plan
}

fn command(task: &veac_codegen::emitter::BackendTask) -> &veac_codegen::emitter::BackendCommand {
    let BackendAction::Ffmpeg(command) = &task.action else {
        panic!()
    };
    command
}

fn requirement(values: &[BackendRequirement], kind: &str, expected: &str) -> bool {
    values
        .iter()
        .any(|value| value.kind().as_str() == kind && value.name() == expected)
}

fn codes(plan: &veac_plan::ResolvedRenderPlan) -> Vec<&'static str> {
    emit_all(plan, &bindings(plan))
        .unwrap_err()
        .diagnostics()
        .iter()
        .map(|value| value.code)
        .collect()
}

fn pair(values: &[String], name: &str, value: &str) -> bool {
    values.windows(2).any(|pair| pair == [name, value])
}

fn pixel_name(value: PixelFormat) -> &'static str {
    match value {
        PixelFormat::Yuv422p => "yuv422p",
        PixelFormat::Yuv422p10le => "yuv422p10le",
        PixelFormat::Yuv444p10le => "yuv444p10le",
        _ => unreachable!(),
    }
}

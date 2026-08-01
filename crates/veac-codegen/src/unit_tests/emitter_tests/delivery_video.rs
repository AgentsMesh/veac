use veac_codegen::emitter::{
    emit_all, BackendAction, BackendCapabilityKind, BackendOutput, BackendPhase, BackendProduct,
};
use veac_plan::canonical::*;

use super::support::{bindings, fixture, resolved};

#[test]
fn two_pass_video_has_exact_phases_and_shared_passlog() {
    let mut plan = resolved(&fixture());
    let video = plan
        .output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap();
    video.pass_mode = PassMode::TwoPass;
    video.hardware = HardwareSelection::Software;
    video.video.rate_control = VideoRateControl::Bitrate {
        target_bps: 1_000_000,
        max_bps: Some(1_500_000),
        buffer_size_bits: Some(2_000_000),
    };
    let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
    assert_eq!(bundle.tasks().len(), 2);
    let first = &bundle.tasks()[0];
    let second = &bundle.tasks()[1];
    assert_eq!(first.phase, BackendPhase::FirstPass);
    assert_eq!(first.product, BackendProduct::RenderPassLog);
    assert_eq!(second.phase, BackendPhase::SecondPass);
    assert_eq!(second.product, BackendProduct::VideoMaster);
    let BackendOutput::File(log) = &first.output else {
        panic!()
    };
    assert_eq!(log.to_string_lossy(), "/tmp/output.mp4.veac-pass-0.log");
    let first = command(first);
    let second = command(second);
    assert!(pair(&first.output_args, "-pass", "1"));
    assert!(pair(&first.output_args, "-f", "null"));
    assert!(first.output_args.iter().any(|value| value == "-an"));
    assert!(pair(&second.output_args, "-pass", "2"));
    assert_eq!(
        option(&first.output_args, "-passlogfile"),
        option(&second.output_args, "-passlogfile")
    );
    for name in ["mp4", "null"] {
        assert!(bundle
            .requirements()
            .iter()
            .any(|value| value.kind() == BackendCapabilityKind::Muxer && value.name() == name));
    }
}

#[test]
fn explicit_hardware_fails_before_emission_without_device_setup() {
    let mut plan = resolved(&fixture());
    plan.output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap()
        .hardware = HardwareSelection::Explicit {
        backend: HardwareBackend::VideoToolbox,
    };
    let error = emit_all(&plan, &bindings(&plan)).unwrap_err();
    assert!(error
        .diagnostics()
        .iter()
        .any(|value| value.code == "PLAN_VIDEO_SETTINGS_INVALID"));
}

#[test]
fn prores_4444_alpha_emits_the_professional_pixel_contract() {
    let mut plan = resolved(&fixture());
    plan.output.deliverables[0].target = DeliverableTarget::File {
        name: "output.mov".to_owned(),
    };
    let video = plan
        .output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap();
    video.container = OutputFormat::Mov;
    video.audio = None;
    video.video = VideoOutput {
        codec: VideoCodec::ProRes,
        pixel_format: PixelFormat::Yuva444p10le,
        alpha: AlphaMode::Straight,
        color_space: None,
        rate_control: VideoRateControl::Lossless,
        gop_size: None,
        b_frames: None,
        profile: Some(VideoProfile::ProRes4444),
        level: None,
    };
    let emitted = emit_all(&plan, &bindings(&plan)).unwrap();
    let arguments = command(&emitted.tasks()[0]).output_args.clone();
    for (name, value) in [
        ("-c:v", "prores_ks"),
        ("-profile:v", "4"),
        ("-pix_fmt", "yuva444p10le"),
        ("-alpha_bits", "16"),
    ] {
        assert!(pair(&arguments, name, value));
    }
}

#[test]
fn pq_and_hlg_emit_ten_bit_bt2020_metadata() {
    for (transfer, expected) in [
        (ColorTransfer::Smpte2084, "smpte2084"),
        (ColorTransfer::AribStdB67, "arib-std-b67"),
    ] {
        let mut plan = resolved(&fixture());
        let video = plan
            .output
            .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
            .unwrap();
        video.video.codec = VideoCodec::H265;
        video.video.pixel_format = PixelFormat::Yuv420p10le;
        video.video.profile = Some(VideoProfile::H265Main10);
        video.video.color_space = Some(ColorSpace {
            primaries: ColorPrimaries::Bt2020,
            transfer,
            matrix: ColorMatrix::Bt2020Ncl,
            range: ColorRange::Limited,
        });
        let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
        let args = &command(&bundle.tasks()[0]).output_args;
        assert!(pair(args, "-pix_fmt", "yuv420p10le"));
        assert!(pair(args, "-color_primaries", "bt2020"));
        assert!(pair(args, "-color_trc", expected));
        assert!(pair(args, "-colorspace", "bt2020nc"));
    }
}

fn command(task: &veac_codegen::emitter::BackendTask) -> &veac_codegen::emitter::BackendCommand {
    let BackendAction::Ffmpeg(command) = &task.action else {
        panic!()
    };
    command
}

fn pair(arguments: &[String], name: &str, value: &str) -> bool {
    arguments.windows(2).any(|pair| pair == [name, value])
}

fn option<'a>(arguments: &'a [String], name: &str) -> Option<&'a str> {
    arguments
        .windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].as_str())
}

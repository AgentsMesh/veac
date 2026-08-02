use veac_codegen::emitter::{emit_all, BackendAction, BackendCapabilityKind, BackendRequirement};
use veac_plan::canonical::*;

use super::support::{bindings, fixture, resolved};

#[test]
fn every_video_encoder_argument_has_an_exact_capability_requirement() {
    for (codec, expected) in [
        (VideoCodec::H264, "libx264"),
        (VideoCodec::H265, "libx265"),
        (VideoCodec::Vp9, "libvpx-vp9"),
        (VideoCodec::Av1, "libaom-av1"),
    ] {
        let mut plan = resolved(&fixture());
        let container = if matches!(codec, VideoCodec::Vp9) {
            OutputFormat::Webm
        } else {
            OutputFormat::Mkv
        };
        plan.output.deliverables[0].target = DeliverableTarget::File {
            name: match container {
                OutputFormat::Webm => "master.webm",
                _ => "master.mkv",
            }
            .to_owned(),
        };
        let delivery = plan
            .output
            .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
            .unwrap();
        delivery.container = container;
        delivery.video.codec = codec;
        delivery.audio = None;
        let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
        assert_encoder(&bundle, "dlv_main", expected);
        assert_requirement(
            &bundle,
            "dlv_main",
            BackendCapabilityKind::Muxer,
            match container {
                OutputFormat::Mp4 => "mp4",
                OutputFormat::Mov => "mov",
                OutputFormat::Mkv => "matroska",
                OutputFormat::Webm => "webm",
                OutputFormat::Mxf => unreachable!(),
            },
        );
        assert!(pair(arguments(&bundle, "dlv_main"), "-c:v", expected));
    }

    let mut plan = resolved(&fixture());
    plan.output.deliverables[0].target = DeliverableTarget::File {
        name: "master.mov".to_owned(),
    };
    let delivery = plan
        .output
        .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
        .unwrap();
    delivery.container = OutputFormat::Mov;
    delivery.audio = None;
    delivery.video = VideoOutput {
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
    let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
    assert_encoder(&bundle, "dlv_main", "prores_ks");
    assert_requirement(&bundle, "dlv_main", BackendCapabilityKind::Muxer, "mov");
}

#[test]
fn every_audio_encoder_argument_has_an_exact_capability_requirement() {
    for (codec, container, expected) in [
        (AudioCodec::Aac, OutputFormat::Mp4, "aac"),
        (AudioCodec::Opus, OutputFormat::Webm, "libopus"),
        (AudioCodec::Flac, OutputFormat::Mkv, "flac"),
        (AudioCodec::PcmS16Le, OutputFormat::Mov, "pcm_s16le"),
        (AudioCodec::PcmS24Le, OutputFormat::Mov, "pcm_s24le"),
        (AudioCodec::PcmS32Le, OutputFormat::Mov, "pcm_s32le"),
    ] {
        let mut plan = resolved(&fixture());
        plan.output.deliverables[0].target = DeliverableTarget::File {
            name: match container {
                OutputFormat::Mp4 => "master.mp4",
                OutputFormat::Mov => "master.mov",
                OutputFormat::Mkv => "master.mkv",
                OutputFormat::Webm => "master.webm",
                OutputFormat::Mxf => unreachable!(),
            }
            .to_owned(),
        };
        let delivery = plan
            .output
            .video_deliverable_mut(&DeliverableId::new("dlv_main").unwrap())
            .unwrap();
        delivery.container = container;
        delivery.video.codec = if container == OutputFormat::Webm {
            VideoCodec::Vp9
        } else {
            VideoCodec::H264
        };
        delivery.audio = Some(AudioOutput {
            codec,
            sample_rate: 48_000,
            channels: 2,
        });
        let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
        assert_encoder(&bundle, "dlv_main", expected);
        assert_requirement(
            &bundle,
            "dlv_main",
            BackendCapabilityKind::Muxer,
            container_name(container),
        );
        assert!(pair(arguments(&bundle, "dlv_main"), "-c:a", expected));
    }
}

#[test]
fn every_image_encoder_argument_has_an_exact_capability_requirement() {
    for (format, extension, expected) in [
        (ImageFormat::Png, "png", "png"),
        (ImageFormat::Jpeg, "jpg", "mjpeg"),
        (ImageFormat::Tiff, "tiff", "tiff"),
        (ImageFormat::Exr, "exr", "exr"),
    ] {
        let mut plan = resolved(&fixture());
        plan.output.deliverables[0].target = DeliverableTarget::ImageSequence {
            pattern: format!("frame-%d.{extension}"),
        };
        plan.output.deliverables[0].kind = DeliverableKind::ImageSequence(ImageSequenceOutput {
            format,
            start_number: 1,
        });
        let bundle = emit_all(&plan, &bindings(&plan)).unwrap();
        assert_encoder(&bundle, "dlv_main", expected);
        assert_requirement(&bundle, "dlv_main", BackendCapabilityKind::Muxer, "image2");
        assert!(pair(arguments(&bundle, "dlv_main"), "-c:v", expected));
    }
}

fn assert_encoder(bundle: &veac_codegen::emitter::BackendBundle, id: &str, expected: &str) {
    assert!(bundle.requirements().iter().any(|value| matches!(
        value,
        BackendRequirement::Encoder { deliverable_id, name }
            if deliverable_id.as_str() == id && name == expected
    )));
}

fn assert_requirement(
    bundle: &veac_codegen::emitter::BackendBundle,
    id: &str,
    kind: BackendCapabilityKind,
    expected: &str,
) {
    assert!(bundle.requirements().iter().any(|value| {
        value.deliverable_id().as_str() == id && value.kind() == kind && value.name() == expected
    }));
}

fn container_name(value: OutputFormat) -> &'static str {
    match value {
        OutputFormat::Mp4 => "mp4",
        OutputFormat::Mov => "mov",
        OutputFormat::Mkv => "matroska",
        OutputFormat::Webm => "webm",
        OutputFormat::Mxf => "mxf",
    }
}

fn arguments<'a>(bundle: &'a veac_codegen::emitter::BackendBundle, id: &str) -> &'a [String] {
    let task = bundle
        .tasks()
        .iter()
        .find(|value| value.deliverable_id.as_str() == id)
        .unwrap();
    let BackendAction::Ffmpeg(command) = &task.action else {
        panic!()
    };
    &command.output_args
}

fn pair(values: &[String], name: &str, value: &str) -> bool {
    values.windows(2).any(|pair| pair == [name, value])
}

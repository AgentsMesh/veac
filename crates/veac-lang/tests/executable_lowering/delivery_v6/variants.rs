use veac_ir::{HardwareBackend, HardwareSelection, OutputFormat, VideoCodec};

use super::super::support;
use super::SOURCE;

#[test]
fn every_video_codec_and_container_reaches_valid_canonical_ir() {
    let cases = [
        (h265(), VideoCodec::H265, OutputFormat::Mp4),
        (vp9(), VideoCodec::Vp9, OutputFormat::Webm),
        (av1(), VideoCodec::Av1, OutputFormat::Mp4),
        (prores(), VideoCodec::ProRes, OutputFormat::Mov),
        (dnxhr(), VideoCodec::DnxHr, OutputFormat::Mxf),
        (mkv(), VideoCodec::H264, OutputFormat::Mkv),
    ];
    for (source, codec, container) in cases {
        let envelope = support::envelope(&source);
        let movie = movie(&envelope);
        assert_eq!((movie.video.codec, movie.container), (codec, container));
        assert!(veac_ir::validate(&envelope).is_ok());
    }
}

#[test]
fn every_hardware_choice_is_a_closed_executable_value() {
    let choices = [
        ("hardware_auto()", HardwareSelection::Auto),
        (
            "hardware_videotoolbox()",
            HardwareSelection::Explicit {
                backend: HardwareBackend::VideoToolbox,
            },
        ),
        (
            "hardware_nvenc()",
            HardwareSelection::Explicit {
                backend: HardwareBackend::Nvenc,
            },
        ),
        (
            "hardware_qsv()",
            HardwareSelection::Explicit {
                backend: HardwareBackend::Qsv,
            },
        ),
        (
            "hardware_vaapi()",
            HardwareSelection::Explicit {
                backend: HardwareBackend::Vaapi,
            },
        ),
    ];
    for (constructor, expected) in choices {
        let source = SOURCE.replace("hardware_software()", constructor);
        let envelope = support::envelope(&source);
        assert_eq!(movie(&envelope).hardware, expected);
    }
}

#[test]
fn two_pass_capped_bitrate_and_embedded_audio_absence_lower_exactly() {
    let source = SOURCE
        .replace(
            "video_crf(20), gop_auto(), b_frames_auto()",
            "video_capped_bitrate(800000, 1000000, 2000000), gop_frames(60), b_frames_count(2)",
        )
        .replace(
            "embedded_audio_present(audio_output(audio_aac(), 48000, 2)),\n      true, pass_single()",
            "embedded_audio_none(),\n      true, pass_two()",
        );
    let envelope = support::envelope(&source);
    let movie = movie(&envelope);
    assert!(movie.audio.is_none());
    assert_eq!(movie.video.gop_size, Some(60));
    assert_eq!(movie.video.b_frames, Some(2));
    assert_eq!(movie.pass_mode, veac_ir::PassMode::TwoPass);
}

fn h265() -> String {
    SOURCE
        .replace(
            "video_h264(), pixel_yuv420p()",
            "video_h265(), pixel_yuv420p10le()",
        )
        .replace("profile_h264_high()", "profile_h265_main10()")
        .replace("h264_level(4, 1)", "h265_level(5, 1)")
}

fn vp9() -> String {
    SOURCE
        .replace(
            "delivery_file(\"main.mp4\")",
            "delivery_file(\"main.webm\")",
        )
        .replace("container_mp4()", "container_webm()")
        .replace("video_h264()", "video_vp9()")
        .replace("profile_h264_high()", "profile_vp9_0()")
        .replace("h264_level(4, 1)", "vp9_level(4, 1)")
        .replace("audio_aac()", "audio_opus()")
        .replace("true, pass_single()", "false, pass_single()")
}

fn av1() -> String {
    SOURCE
        .replace("video_h264()", "video_av1()")
        .replace("profile_h264_high()", "profile_av1_main()")
        .replace("h264_level(4, 1)", "av1_level(5, 1)")
}

fn prores() -> String {
    professional(
        "video_prores()",
        "pixel_yuva444p10le()",
        "alpha_straight()",
        "profile_prores_4444()",
        "container_mov()",
        "main.mov",
    )
}

fn dnxhr() -> String {
    professional(
        "video_dnxhr()",
        "pixel_yuv422p()",
        "alpha_opaque()",
        "profile_dnxhr_hq()",
        "container_mxf()",
        "main.mxf",
    )
    .replace("true, pass_single()", "false, pass_single()")
}

fn professional(
    codec: &str,
    pixel: &str,
    alpha: &str,
    profile: &str,
    container: &str,
    target: &str,
) -> String {
    SOURCE
        .replace(
            "delivery_file(\"main.mp4\")",
            &format!("delivery_file(\"{target}\")"),
        )
        .replace("container_mp4()", container)
        .replace("video_h264()", codec)
        .replace("pixel_yuv420p()", pixel)
        .replace("alpha_opaque()", alpha)
        .replace("video_crf(20)", "video_lossless()")
        .replace("profile_h264_high()", profile)
        .replace(
            "video_level_present(h264_level(4, 1))",
            "video_level_auto()",
        )
        .replace(
            "embedded_audio_present(audio_output(audio_aac(), 48000, 2))",
            "embedded_audio_none()",
        )
}

fn mkv() -> String {
    SOURCE
        .replace("main.mp4", "main.mkv")
        .replace("container_mp4()", "container_mkv()")
        .replace("true, pass_single()", "false, pass_single()")
}

fn movie(envelope: &veac_ir::ProjectEnvelope) -> &veac_ir::VideoDeliverable {
    envelope.project.render_configs[0]
        .deliverables
        .iter()
        .find_map(|artifact| match &artifact.kind {
            veac_ir::DeliverableKind::Video(value) => Some(value),
            _ => None,
        })
        .unwrap()
}

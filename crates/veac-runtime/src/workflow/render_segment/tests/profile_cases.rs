use veac_ir::{AlphaMode, OutputFormat, PixelFormat, VideoCodec, VideoProfile, VideoRateControl};

use super::support::*;
use super::*;

#[test]
fn authored_h264_profile_and_level_match_normalized_ffprobe_facts() {
    let contract = contract_with(false, |delivery| {
        delivery.video.profile = Some(VideoProfile::H264High);
        delivery.video.level = Some("4".to_owned());
    });
    let record = record(&contract);
    let probe = snapshot(&contract, &record);
    validate_full_render_segment_snapshot(&contract, &record, &probe).unwrap();

    let mut wrong_profile = probe.clone();
    wrong_profile.streams[0].video.as_mut().unwrap().profile = Some("Main".to_owned());
    assert_tool_failure(&contract, &record, &wrong_profile);
    let mut wrong_level = probe;
    wrong_level.streams[0].video.as_mut().unwrap().level = Some(41);
    assert_tool_failure(&contract, &record, &wrong_level);
}

#[test]
fn prores_4444_accepts_only_the_controlled_backend_pixel_expansion() {
    let contract = contract_with(false, |delivery| {
        delivery.container = OutputFormat::Mov;
        delivery.video.codec = VideoCodec::ProRes;
        delivery.video.pixel_format = PixelFormat::Yuva444p10le;
        delivery.video.alpha = AlphaMode::Straight;
        delivery.video.rate_control = VideoRateControl::Lossless;
        delivery.video.profile = Some(VideoProfile::ProRes4444);
    });
    let record = record(&contract);
    let mut probe = snapshot(&contract, &record);
    probe.container_brand = Some("qt  ".to_owned());
    let stream = &mut probe.streams[0];
    stream.codec = "prores".to_owned();
    let video = stream.video.as_mut().unwrap();
    video.pixel_format = "yuva444p12le".to_owned();
    video.profile = Some("4444".to_owned());
    video.level = None;
    validate_full_render_segment_snapshot(&contract, &record, &probe).unwrap();

    probe.streams[0].video.as_mut().unwrap().pixel_format = "yuv444p12le".to_owned();
    assert_tool_failure(&contract, &record, &probe);
}

#[test]
fn dnxhr_profile_codec_pixel_and_mxf_family_are_exact() {
    let contract = contract_with(false, |delivery| {
        delivery.container = OutputFormat::Mxf;
        delivery.video.codec = VideoCodec::DnxHr;
        delivery.video.pixel_format = PixelFormat::Yuv422p10le;
        delivery.video.rate_control = VideoRateControl::Lossless;
        delivery.video.profile = Some(VideoProfile::DnxHrHqx);
    });
    let record = record(&contract);
    let mut probe = snapshot(&contract, &record);
    probe.container_format = "mxf".to_owned();
    probe.container_brand = None;
    let stream = &mut probe.streams[0];
    stream.codec = "dnxhd".to_owned();
    let video = stream.video.as_mut().unwrap();
    video.pixel_format = "yuv422p10le".to_owned();
    video.profile = Some("DNXHR HQX".to_owned());
    video.level = None;
    validate_full_render_segment_snapshot(&contract, &record, &probe).unwrap();
}

#[test]
fn identity_bound_ebml_prefix_distinguishes_matroska_from_webm() {
    let mkv = contract_with(false, |delivery| delivery.container = OutputFormat::Mkv);
    validate_full_render_segment_prefix(&mkv, &ebml(b"matroska")).unwrap();
    assert!(validate_full_render_segment_prefix(&mkv, &ebml(b"webm")).is_err());

    let webm = contract_with(false, |delivery| {
        delivery.container = OutputFormat::Webm;
        delivery.video.codec = VideoCodec::Vp9;
    });
    validate_full_render_segment_prefix(&webm, &ebml(b"webm")).unwrap();
    assert!(validate_full_render_segment_prefix(&webm, &ebml(b"matroska")).is_err());
}

fn assert_tool_failure(
    contract: &veac_artifact::FullRenderSegmentContract,
    record: &veac_artifact::ArtifactRecord,
    probe: &veac_ir::MediaProbeSnapshot,
) {
    let error = validate_full_render_segment_snapshot(contract, record, probe).unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
}

fn ebml(document_type: &[u8]) -> Vec<u8> {
    let content = 3 + document_type.len();
    let mut bytes = vec![0x1a, 0x45, 0xdf, 0xa3, 0x80 | content as u8];
    bytes.extend([0x42, 0x82, 0x80 | document_type.len() as u8]);
    bytes.extend(document_type);
    bytes
}

use std::process::Command;

use serde_json::Value;
use tempfile::tempdir;
use veac_artifact::{
    artifact_key, ArtifactRecord, ArtifactStore, ContentDigest, DigestAlgorithm,
    FullRenderSegmentContract,
};
use veac_runtime::executor::{FfmpegEnvironment, SystemFfmpeg};
use veac_runtime::workflow::FullRenderSegmentValidator;

use super::support::*;

#[test]
fn prores_4444_preserves_a_real_transparent_region() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    let deliverable = &mut canonical.project.render_configs[0].deliverables[0];
    deliverable.target = DeliverableTarget::File {
        name: "alpha.mov".to_owned(),
    };
    let DeliverableKind::Video(settings) = &mut deliverable.kind else {
        panic!()
    };
    settings.container = OutputFormat::Mov;
    settings.audio = None;
    settings.video = VideoOutput {
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
    let mut clip = solid_clip("itm_red", color(255, 0, 0), 0, 1_000);
    clip.visual = Some(framed_visual(
        Anchor::Left,
        Some((f64::from(WIDTH / 2), f64::from(HEIGHT))),
    ));
    canonical.project.sequences[0]
        .tracks
        .push(track("trk_red", TrackKind::Video, 0, vec![clip]));
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    let execution = delivery.execute(&ArtifactStore::new(temp.path().join("store")));
    assert_eq!(execution.tasks.len(), 1);
    let output = delivery.path("dlv_main");
    let stream = video_stream(output);
    assert_eq!(stream["codec_name"], "prores");
    assert_profile(&stream["profile"], &["4444", "4"]);
    assert_eq!(stream["pix_fmt"], "yuva444p12le");
    validate_segment(&delivery.plan, output);

    let alpha = raw_alpha(output);
    assert_eq!(alpha.len(), (WIDTH * HEIGHT) as usize);
    let opaque = alpha.iter().filter(|value| **value > 240).count();
    let transparent = alpha.iter().filter(|value| **value < 15).count();
    assert!(opaque > alpha.len() / 5, "opaque pixels {opaque}");
    assert!(
        transparent > alpha.len() / 5,
        "transparent pixels {transparent}"
    );
}

fn validate_segment(plan: &veac_plan::ResolvedRenderPlan, output: &Path) {
    let contract = FullRenderSegmentContract::new(
        plan,
        ContentDigest::sha256(b"alpha source clocks"),
        veac_runtime::workflow::media_artifact_producer(
            &FfmpegEnvironment::fingerprint(&SystemFfmpeg::default()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let identity = veac_runtime::asset::sha256_identity(output).unwrap();
    let record = ArtifactRecord {
        key: artifact_key(contract.descriptor()).unwrap(),
        content: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: identity.digest,
        },
        size_bytes: std::fs::metadata(output).unwrap().len(),
    };
    FullRenderSegmentValidator::new(veac_runtime::asset::SystemFfprobe::default())
        .validate(output, &contract, &record)
        .unwrap();
}

#[test]
fn pq_and_hlg_outputs_carry_real_ten_bit_bt2020_metadata() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_color",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_color", color(80, 120, 160), 0, 1_000)],
    ));
    canonical.project.render_configs[0]
        .raster
        .as_mut()
        .expect("raster fixture")
        .captions = CaptionOutput::Discard;
    canonical.project.render_configs[0].deliverables = vec![
        hdr("dlv_hlg", "hlg.mp4", ColorTransfer::AribStdB67),
        hdr("dlv_pq", "pq.mp4", ColorTransfer::Smpte2084),
    ];
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    delivery.execute(&ArtifactStore::new(temp.path().join("store")));
    for (id, transfer) in [("dlv_hlg", "arib-std-b67"), ("dlv_pq", "smpte2084")] {
        let stream = video_stream(delivery.path(id));
        assert_eq!(stream["codec_name"], "hevc");
        assert_eq!(stream["pix_fmt"], "yuv420p10le");
        assert_eq!(stream["color_primaries"], "bt2020");
        assert_eq!(stream["color_transfer"], transfer);
        assert_eq!(stream["color_space"], "bt2020nc");
    }
}

fn hdr(id: &str, file: &str, transfer: ColorTransfer) -> Deliverable {
    Deliverable {
        id: DeliverableId::new(id).unwrap(),
        target: DeliverableTarget::File {
            name: file.to_owned(),
        },
        kind: DeliverableKind::Video(VideoDeliverable {
            container: OutputFormat::Mp4,
            video: VideoOutput {
                codec: VideoCodec::H265,
                pixel_format: PixelFormat::Yuv420p10le,
                alpha: AlphaMode::Opaque,
                color_space: Some(ColorSpace {
                    primaries: ColorPrimaries::Bt2020,
                    transfer,
                    matrix: ColorMatrix::Bt2020Ncl,
                    range: ColorRange::Limited,
                }),
                rate_control: VideoRateControl::Crf { value: 28 },
                gop_size: None,
                b_frames: None,
                profile: Some(VideoProfile::H265Main10),
                level: None,
            },
            audio: None,
            optimize_for_streaming: false,
            pass_mode: PassMode::Single,
            hardware: HardwareSelection::Software,
        }),
    }
}

fn video_stream(path: &Path) -> Value {
    let output = Command::new("ffprobe")
        .args(["-v", "error", "-select_streams", "v:0", "-show_entries"])
        .arg("stream=codec_name,profile,pix_fmt,color_space,color_transfer,color_primaries")
        .args(["-of", "json"])
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice::<Value>(&output.stdout).unwrap()["streams"][0].clone()
}

fn raw_alpha(path: &Path) -> Vec<u8> {
    let output = Command::new("ffmpeg")
        .args(["-hide_banner", "-loglevel", "error", "-i"])
        .arg(path)
        .args([
            "-vf",
            "alphaextract",
            "-frames:v",
            "1",
            "-pix_fmt",
            "gray",
            "-f",
            "rawvideo",
            "-",
        ])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

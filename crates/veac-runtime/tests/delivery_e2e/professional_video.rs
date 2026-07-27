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
fn dnxhr_hqx_mxf_with_pcm24_is_a_real_probeable_master() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    let output = &mut canonical.project.render_configs[0];
    (output.width, output.height, output.frame_rate) = (256, 120, ratio(24, 1));
    let sequence = &mut canonical.project.sequences[0];
    sequence.settings.width = 256;
    sequence.settings.height = 120;
    sequence.settings.frame_rate = ratio(24, 1);
    sequence.tracks.push(track(
        "trk_source",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_source", color(40, 120, 220), 0, 1_000)],
    ));
    let deliverable = &mut output.deliverables[0];
    deliverable.file_name = "master.mxf".to_owned();
    deliverable.kind = DeliverableKind::Video(VideoDeliverable {
        container: OutputFormat::Mxf,
        video: VideoOutput {
            codec: VideoCodec::DnxHr,
            pixel_format: PixelFormat::Yuv422p10le,
            alpha: AlphaMode::Opaque,
            color_space: None,
            rate_control: VideoRateControl::Lossless,
            gop_size: None,
            b_frames: None,
            profile: Some(VideoProfile::DnxHrHqx),
            level: None,
        },
        audio: Some(AudioOutput {
            codec: AudioCodec::PcmS24Le,
            sample_rate: 48_000,
            channels: 2,
        }),
        captions: CaptionOutput::Discard,
        optimize_for_streaming: false,
        pass_mode: PassMode::Single,
        hardware: HardwareSelection::Software,
    });
    let delivery = prepare_delivery(canonical, &BTreeMap::new(), temp.path());
    let store = ArtifactStore::new(temp.path().join("store"));
    let first = delivery.execute(&store);
    assert_eq!(first.tasks.len(), 1);
    assert!(!first.tasks[0].cache_hit);

    let value = probe(delivery.path("dlv_main"));
    assert!(value["format"]["format_name"]
        .as_str()
        .unwrap()
        .split(',')
        .any(|name| name == "mxf"));
    let streams = value["streams"].as_array().unwrap();
    let video = stream(streams, "video");
    assert_eq!(video["codec_name"], "dnxhd");
    assert_eq!(video["profile"], "DNXHR HQX");
    assert_eq!(video["pix_fmt"], "yuv422p10le");
    let audio = stream(streams, "audio");
    assert_eq!(audio["codec_name"], "pcm_s24le");
    assert_eq!(audio["sample_rate"], "48000");
    assert_eq!(audio["bits_per_raw_sample"], "24");
    let contract = FullRenderSegmentContract::new(
        &delivery.plan,
        ContentDigest::sha256(b"professional source clocks"),
        veac_runtime::workflow::media_artifact_producer(
            &FfmpegEnvironment::fingerprint(&SystemFfmpeg::default()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let identity = veac_runtime::asset::sha256_identity(delivery.path("dlv_main")).unwrap();
    let record = ArtifactRecord {
        key: artifact_key(contract.descriptor()).unwrap(),
        content: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: identity.digest,
        },
        size_bytes: std::fs::metadata(delivery.path("dlv_main")).unwrap().len(),
    };
    FullRenderSegmentValidator::new(veac_runtime::asset::SystemFfprobe::default())
        .validate(delivery.path("dlv_main"), &contract, &record)
        .unwrap();
    assert!(delivery.execute(&store).tasks[0].cache_hit);
}

fn probe(path: &Path) -> Value {
    let output = Command::new("ffprobe")
        .args([
            "-v",
            "error",
            "-show_entries",
            "stream=codec_type,codec_name,profile,pix_fmt,sample_rate,bits_per_raw_sample:format=format_name",
            "-of",
            "json",
        ])
        .arg(path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn stream<'a>(streams: &'a [Value], kind: &str) -> &'a Value {
    streams
        .iter()
        .find(|value| value["codec_type"] == kind)
        .unwrap()
}

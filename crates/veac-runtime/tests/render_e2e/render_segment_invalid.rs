use tempfile::tempdir;
use veac_artifact::{
    artifact_key, ArtifactRecord, ContentDigest, DigestAlgorithm, FullRenderSegmentContract,
};
use veac_runtime::executor::{FfmpegEnvironment, SystemFfmpeg};
use veac_runtime::workflow::{FullRenderSegmentValidator, WorkflowErrorKind};

use super::support::*;

#[test]
fn real_postflight_rejects_wrong_codec_geometry_and_extra_streams() {
    let temp = tempdir().unwrap();
    let mut canonical = project(false);
    canonical.project.sequences[0].tracks.push(track(
        "trk_background",
        TrackKind::Video,
        0,
        vec![solid_clip("itm_background", color(10, 20, 30), 0, 1_000)],
    ));
    let output_id = canonical.project.render_configs[0].id.clone();
    let plan = veac_plan::resolve_one(&canonical, &output_id).unwrap();
    let contract = FullRenderSegmentContract::new(
        &plan,
        ContentDigest::sha256(b"source clocks"),
        veac_runtime::workflow::media_artifact_producer(
            &FfmpegEnvironment::fingerprint(&SystemFfmpeg::default()).unwrap(),
        )
        .unwrap(),
    )
    .unwrap();
    let validator = FullRenderSegmentValidator::new(veac_runtime::asset::SystemFfprobe::default());
    for (name, kind) in [
        ("wrong-codec.mp4", InvalidMedia::Codec),
        ("wrong-geometry.mp4", InvalidMedia::Geometry),
        ("extra-audio.mp4", InvalidMedia::ExtraAudio),
    ] {
        let path = temp.path().join(name);
        invalid_media(&path, kind);
        let record = file_record(&contract, &path);
        let error = validator.validate(&path, &contract, &record).unwrap_err();
        assert_eq!(error.kind, WorkflowErrorKind::ToolFailure, "{name}");
    }
}

fn file_record(contract: &FullRenderSegmentContract, path: &Path) -> ArtifactRecord {
    let identity = veac_runtime::asset::sha256_identity(path).unwrap();
    ArtifactRecord {
        key: artifact_key(contract.descriptor()).unwrap(),
        content: ContentDigest {
            algorithm: DigestAlgorithm::Sha256,
            value: identity.digest,
        },
        size_bytes: std::fs::metadata(path).unwrap().len(),
    }
}

#[derive(Clone, Copy)]
enum InvalidMedia {
    Codec,
    Geometry,
    ExtraAudio,
}

fn invalid_media(path: &Path, kind: InvalidMedia) {
    let geometry = if matches!(kind, InvalidMedia::Geometry) {
        "64x36"
    } else {
        "96x54"
    };
    let mut command = std::process::Command::new("ffmpeg");
    command.args([
        "-hide_banner",
        "-loglevel",
        "error",
        "-f",
        "lavfi",
        "-i",
        &format!("color=c=red:s={geometry}:r=10:d=1"),
    ]);
    if matches!(kind, InvalidMedia::ExtraAudio) {
        command.args(["-f", "lavfi", "-i", "sine=r=48000:d=1", "-c:a", "aac"]);
    }
    command.args([
        "-c:v",
        if matches!(kind, InvalidMedia::Codec) {
            "mpeg4"
        } else {
            "libx264"
        },
        "-pix_fmt",
        "yuv420p",
        "-y",
    ]);
    let output = command.arg(path).output().unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

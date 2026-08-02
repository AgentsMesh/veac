use std::path::{Path, PathBuf};
use std::process::Command;

use veac_artifact::{
    ArtifactDescriptor, ArtifactRecord, ArtifactStore, ContentDigest, MediaArtifactRequest,
    MediaArtifactSpec,
};
use veac_ir::MediaProbeSnapshot;
use veac_runtime::asset::SystemFfprobe;
use veac_runtime::executor::{FfmpegEnvironment, SystemFfmpeg};

pub(super) fn media_fixture(root: &Path) -> PathBuf {
    let output = root.join("source.mp4");
    run(Command::new("ffmpeg")
        .args([
            "-nostdin",
            "-hide_banner",
            "-loglevel",
            "error",
            "-y",
            "-f",
            "lavfi",
            "-i",
            "color=c=red:s=80x60:r=6:d=0.5",
            "-f",
            "lavfi",
            "-i",
            "testsrc2=s=160x90:r=12:d=1",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=220:sample_rate=48000:duration=0.5",
            "-f",
            "lavfi",
            "-i",
            "sine=frequency=880:sample_rate=48000:duration=1",
            "-map",
            "0:v:0",
            "-map",
            "1:v:0",
            "-map",
            "2:a:0",
            "-map",
            "3:a:0",
            "-c:v",
            "libx264",
            "-pix_fmt",
            "yuv420p",
            "-c:a",
            "aac",
            "-t",
            "1",
        ])
        .arg(&output));
    output
}

pub(super) fn request(input: &Path, spec: MediaArtifactSpec) -> MediaArtifactRequest {
    request_with_tool(input, spec, "ffmpeg")
}

pub(super) fn request_with_tool(
    input: &Path,
    spec: MediaArtifactSpec,
    tool: impl Into<PathBuf>,
) -> MediaArtifactRequest {
    let fingerprint = FfmpegEnvironment::fingerprint(&SystemFfmpeg::new(tool)).unwrap();
    MediaArtifactRequest {
        source_identity: ContentDigest::sha256(std::fs::read(input).unwrap()),
        producer: veac_runtime::workflow::media_artifact_producer(&fingerprint).unwrap(),
        spec,
    }
}

pub(super) fn probe_artifact(
    store: &ArtifactStore,
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
) -> MediaProbeSnapshot {
    let artifact = store
        .open_verified(&record.key, descriptor)
        .unwrap()
        .unwrap();
    SystemFfprobe::default()
        .probe(artifact.payload_path())
        .unwrap()
}

pub(super) fn run(command: &mut Command) {
    checked_output(command);
}

pub(super) fn video_frame_count(
    store: &ArtifactStore,
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
) -> u64 {
    let artifact = store
        .open_verified(&record.key, descriptor)
        .unwrap()
        .unwrap();
    let output = checked_output(
        Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-count_frames",
                "-show_entries",
                "stream=nb_read_frames",
                "-of",
                "json",
            ])
            .arg(artifact.payload_path()),
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    value["streams"][0]["nb_read_frames"]
        .as_str()
        .unwrap()
        .parse()
        .unwrap()
}

pub(super) fn first_video_frame_identity(
    store: &ArtifactStore,
    descriptor: &ArtifactDescriptor,
    record: &ArtifactRecord,
) -> ContentDigest {
    let artifact = store
        .open_verified(&record.key, descriptor)
        .unwrap()
        .unwrap();
    let output = checked_output(
        Command::new("ffmpeg")
            .args(["-nostdin", "-v", "error", "-i"])
            .arg(artifact.payload_path())
            .args([
                "-map",
                "0:v:0",
                "-frames:v",
                "1",
                "-pix_fmt",
                "rgb24",
                "-f",
                "rawvideo",
                "pipe:1",
            ]),
    );
    ContentDigest::sha256(output.stdout)
}

fn checked_output(command: &mut Command) -> std::process::Output {
    let output = command.output().expect("start real media tool");
    assert!(
        output.status.success(),
        "tool failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

pub(super) fn cache_directory(root: &Path, key: &ContentDigest) -> PathBuf {
    root.join("sha256")
        .join(&key.value[..2])
        .join(&key.value[2..])
}

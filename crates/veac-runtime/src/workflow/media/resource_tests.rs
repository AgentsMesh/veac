#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};

use veac_artifact::*;

use super::*;

#[test]
fn ffmpeg_output_stderr_and_wall_limits_never_commit() {
    let cases = [
        (
            "output",
            "head -c 2048 /dev/zero > \"$output\"",
            limits(1_024, 5),
        ),
        (
            "stderr",
            "head -c 1048577 /dev/zero >&2\nprintf artifact > \"$output\"",
            limits(4_096, 5),
        ),
        (
            "wall",
            "sleep 5\nprintf artifact > \"$output\"",
            limits(4_096, 1),
        ),
    ];
    for (name, action, policy) in cases {
        let temp = tempfile::tempdir().unwrap();
        let input = source(temp.path());
        let ffmpeg = ffmpeg(temp.path(), name, action);
        let store_root = temp.path().join("store");
        let error = super::test_support::workflow(&ffmpeg, temp.path())
            .with_limits(policy)
            .derive(&ArtifactStore::new(&store_root), &input, &request(&ffmpeg))
            .unwrap_err();
        assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit, "{name}");
        assert!(!store_root.exists(), "{name}");
    }
}

#[test]
fn preflight_and_postflight_share_one_wall_deadline() {
    let temp = tempfile::tempdir().unwrap();
    let input = source(temp.path());
    let ffmpeg = ffmpeg(temp.path(), "fast", "printf artifact > \"$output\"");
    let store_root = temp.path().join("store");
    let error = MediaWorkflow::with_tools(&ffmpeg, slow_ffprobe(temp.path()))
        .with_limits(limits(4_096, 1))
        .derive(&ArtifactStore::new(&store_root), &input, &request(&ffmpeg))
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(!store_root.exists());
}

#[test]
fn active_work_limit_rejects_before_tools_or_store() {
    let temp = tempfile::tempdir().unwrap();
    let input = source(temp.path());
    let marker = temp.path().join("executed");
    let ffmpeg = ffmpeg(
        temp.path(),
        "marker",
        &format!("touch '{}'", marker.display()),
    );
    let mut policy = limits(4_096, 5);
    policy.max_audio_channel_samples = 1;
    let store_root = temp.path().join("store");
    let error = super::test_support::workflow(&ffmpeg, temp.path())
        .with_limits(policy)
        .derive(&ArtifactStore::new(&store_root), &input, &request(&ffmpeg))
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(!marker.exists());
    assert!(!store_root.exists());
}

fn limits(max_payload_bytes: u64, max_wall_seconds: u64) -> MediaArtifactLimits {
    MediaArtifactLimits {
        max_payload_bytes,
        max_derivation_wall_seconds: max_wall_seconds,
        ..MediaArtifactLimits::default()
    }
}

fn source(root: &Path) -> PathBuf {
    let path = root.join("input");
    fs::write(&path, b"source").unwrap();
    path
}

fn request(tool: &Path) -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity: ContentDigest::sha256(b"source"),
        producer: super::security_tests::ffmpeg_producer(tool),
        spec: MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: veac_ir::StreamSelection {
                global_index: 0,
                type_index: 0,
            },
            source_clock: SourceClockSpec::Identity {
                duration: veac_ir::RationalTime::new(1, 1).unwrap(),
            },
            sample_rate: 48_000,
            channels: 2,
        }),
    }
}

fn ffmpeg(root: &Path, name: &str, action: &str) -> PathBuf {
    super::test_support::executable(
        root.join(format!("ffmpeg-{name}.sh")),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffmpeg version limits'; exit 0; fi\nfor output in \"$@\"; do :; done\n{action}\n"
        ),
    )
}

fn slow_ffprobe(root: &Path) -> PathBuf {
    super::test_support::executable(
        root.join("slow-ffprobe.sh"),
        "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffprobe version slow'; exit 0; fi\nsleep 0.65\nprintf '%s' '{\"format\":{\"format_name\":\"wav\",\"duration\":\"1\"},\"streams\":[{\"index\":0,\"codec_type\":\"audio\",\"codec_name\":\"pcm_s16le\",\"time_base\":\"1/48000\",\"start_time\":\"0\",\"duration\":\"1\",\"sample_rate\":\"48000\",\"channels\":2,\"channel_layout\":\"stereo\"}]}'\n",
    )
}

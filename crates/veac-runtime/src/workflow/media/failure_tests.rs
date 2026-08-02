#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use veac_artifact::{
    ArtifactStore, ContentDigest, MediaArtifactRequest, MediaArtifactSpec, ProxyAudioSpec,
    SourceClockSpec,
};

use super::*;

#[test]
fn derivation_reports_process_failures_after_verification() {
    let temp = tempfile::tempdir().unwrap();
    let input = source(temp.path());

    let failing = tool(temp.path(), "failing", "printf 'render failed' >&2\nexit 9");
    let request = audio_request(&failing);
    let error = super::test_support::workflow(&failing, temp.path())
        .derive(&store(temp.path(), "failure-store"), &input, &request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
    assert!(error.to_string().contains("render failed"));
}

#[test]
fn derivation_rejects_missing_and_non_regular_tool_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let input = source(temp.path());

    let missing = tool(temp.path(), "missing-output", "exit 0");
    let request = audio_request(&missing);
    let error = super::test_support::workflow(&missing, temp.path())
        .derive(&store(temp.path(), "missing-store"), &input, &request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
    assert!(error.to_string().contains("did not produce"));

    let empty = tool(temp.path(), "empty-output", ": > \"$output\"");
    let request = audio_request(&empty);
    let error = super::test_support::workflow(&empty, temp.path())
        .derive(&store(temp.path(), "empty-store"), &input, &request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
    assert!(error.to_string().contains("regular non-empty"));
}

fn source(root: &Path) -> PathBuf {
    let path = root.join("input.bin");
    fs::write(&path, b"source").unwrap();
    path
}

fn store(root: &Path, name: &str) -> ArtifactStore {
    ArtifactStore::new(root.join(name))
}

fn audio_request(binary: &Path) -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity: ContentDigest::sha256(b"source"),
        producer: super::security_tests::ffmpeg_producer(binary),
        spec: MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: veac_ir::StreamSelection {
                global_index: 0,
                type_index: 0,
            },
            source_clock: SourceClockSpec::Identity {
                duration: veac_ir::RationalTime::new(1_000, 1_000).unwrap(),
            },
            sample_rate: 48_000,
            channels: 2,
        }),
    }
}

fn tool(root: &Path, name: &str, action: &str) -> PathBuf {
    executable(
        root.join(format!("{name}.sh")),
        &format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  printf 'ffmpeg version failure-contract-v1\\n'\n  exit 0\nfi\nfor output in \"$@\"; do :; done\n{action}\n"
        ),
    )
}

fn executable(path: PathBuf, contents: &str) -> PathBuf {
    fs::write(&path, contents).unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

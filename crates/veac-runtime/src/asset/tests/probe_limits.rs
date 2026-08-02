#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use super::*;

#[test]
fn version_and_probe_output_streams_are_bounded() {
    let temp = tempfile::tempdir().unwrap();
    let media = media(temp.path());
    let version = script(
        temp.path(),
        "version-output.sh",
        "head -c 5000 /dev/zero | tr '\\0' x",
        "exit 0",
    );
    assert_limit(&media, &version);

    let stdout = script(
        temp.path(),
        "probe-output.sh",
        "echo 'ffprobe version bounded'",
        "head -c 1048577 /dev/zero",
    );
    assert_limit(&media, &stdout);

    let stderr = script(
        temp.path(),
        "probe-stderr.sh",
        "echo 'ffprobe version bounded'",
        "head -c 1048577 /dev/zero >&2",
    );
    assert_limit(&media, &stderr);
}

#[test]
fn version_and_probe_share_a_bounded_wall_deadline() {
    let temp = tempfile::tempdir().unwrap();
    let media = media(temp.path());
    let version = script(temp.path(), "version-hang.sh", "sleep 5", "exit 0");
    assert_limit(&media, &version);

    let probe = script(
        temp.path(),
        "probe-hang.sh",
        "echo 'ffprobe version bounded'",
        "sleep 5",
    );
    assert_limit(&media, &probe);
}

#[test]
fn invalid_bounded_probe_policy_fails_before_execution() {
    let temp = tempfile::tempdir().unwrap();
    let media = media(temp.path());
    let marker = temp.path().join("should-not-exist");
    let action = format!("touch '{}'", marker.display());
    let tool = script(temp.path(), "marker.sh", &action, &action);
    let error = SystemFfprobe::new(tool)
        .probe_with_intent_bounded(&media, auto_stream_intent(), 0)
        .unwrap_err();
    assert!(matches!(error, ProbeError::ResourceLimit { .. }));
    assert!(!marker.exists());
}

fn assert_limit(media: &Path, binary: &Path) {
    let error = SystemFfprobe::new(binary)
        .probe_with_intent_bounded(media, auto_stream_intent(), 1)
        .unwrap_err();
    assert!(matches!(error, ProbeError::ResourceLimit { .. }));
    assert!(error.to_string().contains("resource budget"));
}

fn media(root: &Path) -> PathBuf {
    let path = root.join("media.bin");
    std::fs::write(&path, b"media").unwrap();
    path
}

fn script(root: &Path, name: &str, version: &str, probe: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(
        &path,
        format!("#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n{version}\nexit 0\nfi\n{probe}\n"),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

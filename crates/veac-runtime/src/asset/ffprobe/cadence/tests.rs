#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use tempfile::tempdir;
use veac_ir::{StreamChoice, StreamIntent};

use super::*;
use crate::asset::tests::{complete_output, parse};

#[test]
fn timeout_and_output_limit_are_attempted_without_evidence() {
    let temp = tempdir().unwrap();
    let source = media(temp.path());
    let snapshot = parse(
        &complete_output(),
        StreamIntent {
            video: StreamChoice::Auto,
            audio: StreamChoice::Auto,
        },
    )
    .unwrap();
    let timeout = script(temp.path(), "timeout.sh", "sleep 30");
    assert!(matches!(
        inspect_with_limit(
            &snapshot,
            &timeout,
            &source,
            Instant::now() + Duration::from_millis(100),
            4_096,
        ),
        Inspection::Attempted(None)
    ));

    let overflow = script(temp.path(), "overflow.sh", "head -c 4097 /dev/zero");
    assert!(matches!(
        inspect_with_limit(
            &snapshot,
            &overflow,
            &source,
            Instant::now() + Duration::from_secs(30),
            4_096,
        ),
        Inspection::Attempted(None)
    ));
}

#[test]
fn absent_video_selection_skips_the_optional_process() {
    let temp = tempdir().unwrap();
    let source = media(temp.path());
    let marker = temp.path().join("cadence-ran");
    let command = format!("printf ran > '{}'", marker.display());
    let probe = script(temp.path(), "skipped.sh", &command);
    let snapshot = parse(
        &complete_output(),
        StreamIntent {
            video: StreamChoice::Disabled,
            audio: StreamChoice::Auto,
        },
    )
    .unwrap();

    assert!(matches!(
        inspect_with_limit(
            &snapshot,
            &probe,
            &source,
            Instant::now() + Duration::from_secs(30),
            4_096,
        ),
        Inspection::Skipped
    ));
    assert!(!marker.exists());
}

fn media(root: &Path) -> PathBuf {
    let path = root.join("media.bin");
    std::fs::write(&path, b"source").unwrap();
    path
}

fn script(root: &Path, name: &str, command: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{command}\n")).unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

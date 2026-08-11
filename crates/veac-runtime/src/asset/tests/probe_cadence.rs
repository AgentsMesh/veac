#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use tempfile::tempdir;

use super::*;

#[test]
fn optional_cadence_process_failure_does_not_kill_a_forward_probe() {
    let temp = tempdir().unwrap();
    let media = media(temp.path());
    let probe = script(temp.path(), "cadence-fails.sh", "exit 9");

    assert_unknown(&run(&media, &probe, 30));
}

#[test]
fn malformed_optional_cadence_evidence_remains_unknown() {
    let temp = tempdir().unwrap();
    let media = media(temp.path());
    let probe = script(temp.path(), "cadence-malformed.sh", "printf not-json");

    assert_unknown(&run(&media, &probe, 30));
}

#[test]
fn failed_optional_cadence_still_detects_source_identity_drift() {
    let temp = tempdir().unwrap();
    let media = media(temp.path());
    let cadence = format!("printf changed > '{}'; exit 9", media.display());
    let probe = script(temp.path(), "cadence-source-drift.sh", &cadence);

    let error = SystemFfprobe::new(&probe)
        .probe_with_intent_bounded(&media, auto_stream_intent(), 30)
        .unwrap_err();
    assert!(matches!(error, ProbeError::IdentityChanged { .. }));
}

fn run(media: &Path, probe: &Path, seconds: u64) -> MediaProbeSnapshot {
    SystemFfprobe::new(probe)
        .probe_with_intent_bounded(media, auto_stream_intent(), seconds)
        .unwrap()
}

fn assert_unknown(snapshot: &MediaProbeSnapshot) {
    let selection = snapshot.selected_video_stream.unwrap();
    let info = snapshot
        .streams
        .iter()
        .find(|stream| stream.global_index == selection.global_index)
        .unwrap()
        .video
        .as_ref()
        .unwrap();
    assert_eq!(info.cadence, veac_ir::VideoCadence::Unknown);
}

fn media(root: &Path) -> PathBuf {
    let path = root.join("media.bin");
    std::fs::write(&path, b"original").unwrap();
    path
}

fn script(root: &Path, name: &str, cadence: &str) -> PathBuf {
    let path = root.join(name);
    let output = complete_output().to_string();
    std::fs::write(
        &path,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffprobe version test-1'; exit 0; fi\ncase \" $* \" in *\" -show_packets \"*) {cadence}; exit 0;; esac\nprintf '%s' '{output}'\n"
        ),
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

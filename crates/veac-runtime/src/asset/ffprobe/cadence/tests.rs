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
    let source = captured_media(temp.path());
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
    let source = captured_media(temp.path());
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

#[test]
fn missing_video_timing_skips_and_absent_selection_ignores_evidence() {
    let temp = tempdir().unwrap();
    let source = captured_media(temp.path());
    let probe = script(temp.path(), "unused.sh", "exit 99");
    let intent = StreamIntent {
        video: StreamChoice::Auto,
        audio: StreamChoice::Auto,
    };
    let mut snapshot = parse(&complete_output(), intent.clone()).unwrap();
    let selected = snapshot.selected_video_stream.unwrap().global_index;
    snapshot
        .streams
        .iter_mut()
        .find(|stream| stream.global_index == selected)
        .unwrap()
        .time_base = None;
    assert!(matches!(
        inspect(
            &snapshot,
            &probe,
            &source,
            Instant::now() + Duration::from_secs(1)
        ),
        Inspection::Skipped
    ));

    snapshot = parse(&complete_output(), intent).unwrap();
    snapshot
        .streams
        .iter_mut()
        .find(|stream| stream.global_index == selected)
        .unwrap()
        .video = None;
    assert!(matches!(
        inspect(
            &snapshot,
            &probe,
            &source,
            Instant::now() + Duration::from_secs(1)
        ),
        Inspection::Skipped
    ));

    snapshot.selected_video_stream = None;
    apply(
        &mut snapshot,
        Evidence {
            cadence: VideoCadence::Constant,
            frame_rate: Some(Rational::new(24, 1).unwrap()),
        },
    );
    assert!(snapshot.streams.iter().all(|stream| stream.video.is_none()));
}

#[test]
fn cadence_arguments_reuse_the_snapshot_image_demuxer() {
    let temp = tempdir().unwrap();
    let path = temp.path().join("frame.PNG");
    std::fs::write(&path, b"snapshot bytes").unwrap();
    let source = ProbeSource::capture(&path, Instant::now() + Duration::from_secs(3)).unwrap();
    let values = arguments(&source, 2);
    let input = values.iter().position(|value| value == "-i").unwrap();

    assert_eq!(values[input - 2], "-f");
    assert_eq!(values[input - 1], "png_pipe");
    let snapshot = Path::new(values[input + 1].as_os_str());
    assert_eq!(snapshot.file_name().unwrap(), "media.PNG");
    assert_ne!(snapshot, path);
    assert!(values
        .windows(2)
        .any(|pair| pair == ["-select_streams", "v:2"]));
}

fn media(root: &Path) -> PathBuf {
    let path = root.join("media.bin");
    std::fs::write(&path, b"source").unwrap();
    path
}

fn captured_media(root: &Path) -> ProbeSource {
    ProbeSource::capture(&media(root), Instant::now() + Duration::from_secs(3)).unwrap()
}

fn script(root: &Path, name: &str, command: &str) -> PathBuf {
    let path = root.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{command}\n")).unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o755);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

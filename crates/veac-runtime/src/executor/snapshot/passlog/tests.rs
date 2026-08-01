use veac_artifact::{ArtifactRecord, ContentDigest};
use veac_codegen::emitter::BackendAction;

use super::*;
use crate::executor::tests::support::{two_pass_tasks, video_task};
use crate::RuntimeErrorKind;

#[test]
fn expired_passlog_copy_preserves_resource_limit_kind() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output.mp4");
    let task = two_pass_tasks("master", &output).pop().unwrap();
    let path = temp.path().join("output.mp4.veac-pass-0.log");
    std::fs::write(&path, b"log").unwrap();

    let error = ReboundPasslogs::capture(
        &task,
        std::slice::from_ref(&path),
        std::slice::from_ref(&path),
        &[record(b"log")],
        std::time::Instant::now(),
    )
    .unwrap_err();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
}

#[test]
fn rejects_incomplete_and_non_second_pass_contracts() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output.mp4");
    let task = two_pass_tasks("master", &output).pop().unwrap();
    let error = ReboundPasslogs::capture(&task, &[], &[], &[], deadline()).unwrap_err();
    assert!(error
        .message
        .contains("complete checkpointed passlog outputs"));

    let record = record(b"log");
    let path = temp.path().join("output.mp4.veac-pass-0.log");
    std::fs::write(&path, b"log").unwrap();
    let single = video_task("master", &output);
    assert!(ReboundPasslogs::capture(
        &single,
        std::slice::from_ref(&path),
        std::slice::from_ref(&path),
        std::slice::from_ref(&record),
        deadline(),
    )
    .is_err());

    let mut not_ffmpeg = two_pass_tasks("master", &output).pop().unwrap();
    not_ffmpeg.action = BackendAction::WriteFile {
        path: output,
        content: b"invalid".to_vec(),
    };
    assert!(ReboundPasslogs::capture(
        &not_ffmpeg,
        std::slice::from_ref(&path),
        std::slice::from_ref(&path),
        std::slice::from_ref(&record),
        deadline(),
    )
    .unwrap_err()
    .message
    .contains("FFmpeg action"));
}

#[test]
fn rejects_wrong_family_size_and_identity() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output.mp4");
    let task = two_pass_tasks("master", &output).pop().unwrap();
    let foreign = temp.path().join("foreign.log");
    std::fs::write(&foreign, b"log").unwrap();
    assert!(ReboundPasslogs::capture(
        &task,
        std::slice::from_ref(&foreign),
        std::slice::from_ref(&foreign),
        &[record(b"log")],
        deadline(),
    )
    .unwrap_err()
    .message
    .contains("passlog family"));

    let log = temp.path().join("output.mp4.veac-pass-0.log");
    std::fs::write(&log, b"log").unwrap();
    let mut wrong_size = record(b"log");
    wrong_size.size_bytes += 1;
    assert!(ReboundPasslogs::capture(
        &task,
        std::slice::from_ref(&log),
        std::slice::from_ref(&log),
        &[wrong_size],
        deadline(),
    )
    .unwrap_err()
    .message
    .contains("size changed"));
    assert!(ReboundPasslogs::capture(
        &task,
        std::slice::from_ref(&log),
        std::slice::from_ref(&log),
        &[record(b"other")],
        deadline(),
    )
    .unwrap_err()
    .message
    .contains("identity"));
}

fn record(bytes: &[u8]) -> ArtifactRecord {
    ArtifactRecord {
        key: ContentDigest::sha256(b"key"),
        content: ContentDigest::sha256(bytes),
        size_bytes: bytes.len() as u64,
    }
}

fn deadline() -> std::time::Instant {
    std::time::Instant::now() + std::time::Duration::from_secs(10)
}

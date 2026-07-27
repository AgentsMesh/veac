use std::time::{Duration, Instant};

use super::test_support::audio_spec;
use super::*;

#[test]
fn probe_wrapper_accepts_a_valid_audio_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = temp.path().join("artifact.wav");
    std::fs::write(&artifact, b"artifact").unwrap();
    let tool = SystemFfprobe::new(executable(
        temp.path().join("ffprobe"),
        r#"#!/bin/sh
if [ "$1" = "-version" ]; then printf 'ffprobe version postflight\n'; exit 0; fi
printf '%s' '{"format":{"format_name":"wav","duration":"1"},"streams":[{"index":0,"codec_type":"audio","codec_name":"pcm_s16le","time_base":"1/48000","duration":"1","sample_rate":"48000","channels":2,"channel_layout":"stereo"}]}'
"#,
    ));
    let result = validate(&tool, &artifact, &audio_spec(), future());
    if let Err(error) = result {
        panic!("{error}");
    }
}

#[test]
fn probe_wrapper_preserves_deadline_and_tool_failure_kinds() {
    let temp = tempfile::tempdir().unwrap();
    let artifact = temp.path().join("artifact.wav");
    std::fs::write(&artifact, b"artifact").unwrap();
    let missing = SystemFfprobe::new(temp.path().join("missing"));
    assert_eq!(
        validate(&missing, &artifact, &audio_spec(), Instant::now())
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ResourceLimit
    );
    assert_eq!(
        validate(&missing, &artifact, &audio_spec(), future())
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ToolFailure
    );
}

fn executable(path: std::path::PathBuf, contents: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;
    std::fs::write(&path, contents).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}

fn future() -> Instant {
    Instant::now() + Duration::from_secs(30)
}

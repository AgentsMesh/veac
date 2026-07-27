use std::fs;
use std::path::{Path, PathBuf};

use veac_artifact::*;

use super::*;

#[cfg(unix)]
#[test]
fn wrong_ffmpeg_fingerprints_never_execute_or_touch_the_store() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fs::write(&input, b"source").unwrap();
    let tool = fake_ffmpeg(temp.path(), false);
    let actual = ffmpeg_producer(&tool);
    let mut wrong_name = actual.clone();
    wrong_name.name = "other".into();
    let mut wrong_version = actual.clone();
    wrong_version.version = "other".into();
    let mut wrong_configuration = actual;
    wrong_configuration.configuration = ContentDigest::sha256(b"other");

    for (index, fingerprint) in [wrong_name, wrong_version, wrong_configuration]
        .into_iter()
        .enumerate()
    {
        let root = temp.path().join(format!("store-{index}"));
        let request = audio_request(ContentDigest::sha256(b"source"), fingerprint);
        let error = MediaWorkflow::new(&tool)
            .derive(&ArtifactStore::new(&root), &input, &request)
            .unwrap_err();
        assert_eq!(error.kind, WorkflowErrorKind::InvalidContract);
        assert!(!root.exists());
        assert!(!action_marker(&tool).exists());
    }
}

#[cfg(unix)]
#[test]
fn same_version_different_ffmpeg_bytes_are_not_the_same_producer() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fs::write(&input, b"source").unwrap();
    let expected_tool = fake_ffmpeg(temp.path(), false);
    let other_tool = fake_ffmpeg(temp.path(), true);
    let request = audio_request(
        ContentDigest::sha256(b"source"),
        ffmpeg_producer(&expected_tool),
    );
    let root = temp.path().join("store");

    let error = MediaWorkflow::new(&other_tool)
        .derive(&ArtifactStore::new(&root), &input, &request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::InvalidContract);
    assert_eq!(fs::read(input).unwrap(), b"source");
    assert!(!action_marker(&other_tool).exists());
    assert!(!root.exists());
}

#[cfg(unix)]
#[test]
fn source_replaced_during_ffmpeg_execution_is_never_committed() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fs::write(&input, b"source").unwrap();
    let tool = fake_ffmpeg(temp.path(), true);
    let request = audio_request(ContentDigest::sha256(b"source"), ffmpeg_producer(&tool));
    let root = temp.path().join("store");

    let error = super::test_support::workflow(&tool, temp.path())
        .derive(&ArtifactStore::new(&root), &input, &request)
        .unwrap_err();

    assert_eq!(error.kind, WorkflowErrorKind::SourceIdentityMismatch);
    assert_eq!(fs::read(&input).unwrap(), b"change");
    assert!(action_marker(&tool).exists());
    assert!(!root.exists());
}

#[cfg(unix)]
#[test]
fn transient_source_and_tool_swaps_still_consume_both_pinned_snapshots() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fs::write(&input, b"source-a").unwrap();
    let tool = swapping_ffmpeg(temp.path(), &input);
    let request = audio_request(ContentDigest::sha256(b"source-a"), ffmpeg_producer(&tool));
    let store = ArtifactStore::new(temp.path().join("store"));

    let created = super::test_support::workflow(&tool, temp.path())
        .derive(&store, &input, &request)
        .unwrap();

    assert_eq!(
        store.get(&created.record.key).unwrap().unwrap().payload,
        b"source-a"
    );
    assert_eq!(fs::read(&input).unwrap(), b"source-a");
    assert!(fs::read_to_string(&tool).unwrap().contains("exit 97"));
}

fn audio_request(
    source_identity: ContentDigest,
    producer: ProducerFingerprint,
) -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity,
        producer,
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

pub(super) fn ffmpeg_producer(path: &Path) -> ProducerFingerprint {
    use crate::executor::{FfmpegEnvironment, SystemFfmpeg};
    use crate::workflow::media_artifact_producer;

    let value = FfmpegEnvironment::fingerprint(&SystemFfmpeg::new(path)).unwrap();
    media_artifact_producer(&value).unwrap()
}

#[cfg(unix)]
fn fake_ffmpeg(root: &Path, replace_source: bool) -> PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = root.join(if replace_source {
        "replacing-ffmpeg.sh"
    } else {
        "stable-ffmpeg.sh"
    });
    let replacement = if replace_source {
        format!("printf change > '{}'\n", root.join("input").display())
    } else {
        String::new()
    };
    let marker = action_marker(&path);
    fs::write(
        &path,
        format!(
            "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  printf 'ffmpeg version fake-security-v1\\n'\n  exit 0\nfi\nprintf invoked > '{}'\ninput=\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-i\" ]; then shift; input=$1; fi\n  output=$1\n  shift\ndone\n{replacement}printf artifact > \"$output\"\n",
            marker.display(),
        ),
    )
    .unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

#[cfg(unix)]
fn swapping_ffmpeg(root: &Path, source: &Path) -> PathBuf {
    let path = root.join("swapping-ffmpeg.sh");
    let marker = root.join("versioned");
    let replacement = "#!/bin/sh\nexit 97\n";
    let contents = format!(
        "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  if [ -e '{}' ]; then printf '{}' > '{}'; chmod 700 '{}'; fi\n  : > '{}'\n  printf 'ffmpeg version pinned-media-v1\\n'\n  rm \"$0\"\n  exit 0\nfi\nprintf source-b > '{}'\ninput=\nwhile [ \"$#\" -gt 0 ]; do\n  if [ \"$1\" = \"-i\" ]; then shift; input=$1; fi\n  output=$1\n  shift\ndone\ncat \"$input\" > \"$output\"\nprintf source-a > '{}'\nrm \"$0\"\n",
        marker.display(),
        replacement,
        path.display(),
        path.display(),
        marker.display(),
        source.display(),
        source.display(),
    );
    fs::write(&path, contents).unwrap();
    use std::os::unix::fs::PermissionsExt;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    path
}

fn action_marker(tool: &Path) -> PathBuf {
    PathBuf::from(format!("{}.action", tool.display()))
}

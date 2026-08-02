use std::fs;

use serde_json::json;
use veac_artifact::*;

use super::*;

#[test]
fn media_workflow_rejects_wrong_sources_and_implicit_analysis() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fs::write(&input, b"source").unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let workflow = MediaWorkflow::new("unused");
    let mut request = analysis_request(ContentDigest::sha256(b"wrong"));
    assert_eq!(
        workflow
            .store_analysis(&store, &input, &request, &json!({}))
            .unwrap_err()
            .kind,
        WorkflowErrorKind::SourceIdentityMismatch
    );
    request.source_identity = ContentDigest::sha256(b"source");
    assert_eq!(
        workflow.derive(&store, &input, &request).unwrap_err().kind,
        WorkflowErrorKind::UnsupportedOperation
    );
    assert_eq!(
        workflow
            .store_analysis(&store, &input, &request, &json!([]))
            .unwrap_err()
            .kind,
        WorkflowErrorKind::UnsupportedOperation
    );
}

#[test]
fn media_workflow_reports_tool_start_and_missing_output() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fs::write(&input, b"source").unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let request = audio_request(ContentDigest::sha256(b"source"));
    assert_eq!(
        MediaWorkflow::new(temp.path().join("missing"))
            .derive(&store, &input, &request)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ToolFailure
    );
    assert_eq!(
        MediaWorkflow::new("/usr/bin/true")
            .derive(&store, &input, &request)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ToolFailure
    );
    assert_eq!(
        MediaWorkflow::new("/usr/bin/false")
            .derive(&store, &input, &request)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ToolFailure
    );
}

#[cfg(unix)]
#[test]
fn media_workflow_rejects_symlink_sources() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    let linked = temp.path().join("linked");
    fs::write(&input, b"source").unwrap();
    symlink(&input, &linked).unwrap();
    let error = MediaWorkflow::new("unused")
        .derive(
            &ArtifactStore::new(temp.path().join("store")),
            &linked,
            &audio_request(ContentDigest::sha256(b"source")),
        )
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::InvalidContract);
}

#[cfg(unix)]
#[test]
fn media_workflow_commits_and_reuses_derived_and_analysis_artifacts() {
    let temp = tempfile::tempdir().unwrap();
    let input = temp.path().join("input");
    fs::write(&input, b"source").unwrap();
    let identity = ContentDigest::sha256(b"source");
    let store = ArtifactStore::new(temp.path().join("store"));
    let tool = fake_ffmpeg(temp.path());
    let workflow = super::test_support::workflow(&tool, temp.path());

    let mut request = audio_request(identity.clone());
    request.producer = super::security_tests::ffmpeg_producer(&tool);
    let created = workflow.derive(&store, &input, &request).unwrap();
    assert!(!created.cache_hit);
    assert_eq!(
        store.get(&created.record.key).unwrap().unwrap().payload,
        b"artifact"
    );
    let cached = workflow.derive(&store, &input, &request).unwrap();
    assert!(cached.cache_hit);
    assert_eq!(cached.record, created.record);

    let request = analysis_request(identity);
    let created = workflow
        .store_analysis(
            &store,
            &input,
            &request,
            &json!({"score": 1, "label": "cut"}),
        )
        .unwrap();
    assert!(!created.cache_hit);
    assert_eq!(
        store.get(&created.record.key).unwrap().unwrap().payload,
        br#"{"label":"cut","score":1}"#
    );
    let cached = workflow
        .store_analysis(
            &store,
            &input,
            &request,
            &json!({"score": 1, "label": "cut"}),
        )
        .unwrap();
    assert!(cached.cache_hit);
    assert_eq!(cached.record, created.record);
}

#[cfg(unix)]
fn fake_ffmpeg(root: &std::path::Path) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = root.join("fake-ffmpeg.sh");
    fs::write(
        &path,
        "#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then\n  printf 'ffmpeg version fake-media-v1\\n'\n  exit 0\nfi\nfor output in \"$@\"; do :; done\nprintf artifact > \"$output\"\n",
    )
    .unwrap();
    let mut permissions = fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&path, permissions).unwrap();
    path
}

fn analysis_request(source_identity: ContentDigest) -> MediaArtifactRequest {
    request(
        source_identity,
        MediaArtifactSpec::Analysis(AnalysisSpec {
            analysis_type: "scenes".into(),
            configuration: json!({}),
        }),
    )
}

fn audio_request(source_identity: ContentDigest) -> MediaArtifactRequest {
    request(
        source_identity,
        MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: veac_ir::StreamSelection {
                global_index: 0,
                type_index: 0,
            },
            source_clock: veac_artifact::SourceClockSpec::Identity {
                duration: veac_ir::RationalTime::new(1_000, 1_000).unwrap(),
            },
            sample_rate: 48_000,
            channels: 2,
        }),
    )
}

fn request(source_identity: ContentDigest, spec: MediaArtifactSpec) -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity,
        producer: ProducerFingerprint {
            name: "test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
        spec,
    }
}

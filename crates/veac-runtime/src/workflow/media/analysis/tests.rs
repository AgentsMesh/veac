use serde_json::json;
use veac_artifact::*;

use super::*;

#[test]
fn expired_analysis_deadline_fails_before_source_or_cache_access() {
    let temp = tempfile::tempdir().unwrap();
    let artifact_store = ArtifactStore::new(temp.path().join("store"));
    let request = request(ContentDigest::sha256(b"source"));
    let error = super::store(
        &artifact_store,
        &temp.path().join("missing"),
        &request,
        &json!({"score": 1}),
        MediaArtifactLimits::default(),
        Instant::now(),
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(!artifact_store.root().exists());
}

#[test]
fn cached_analysis_read_obeys_its_guard() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = request(ContentDigest::sha256(b"source"))
        .descriptor()
        .unwrap();
    let record = store.put(&descriptor, br#"{"score":1}"#).unwrap();
    let artifact = store
        .open_verified(&record.key, &descriptor)
        .unwrap()
        .unwrap();
    let error = cache::validate(&artifact, 1_024, &mut || false).unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
}

#[test]
fn fresh_analysis_store_obeys_its_guard_without_publication() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = request(ContentDigest::sha256(b"source"))
        .descriptor()
        .unwrap();
    let key = artifact_key(&descriptor).unwrap();
    let error = store_fresh(&store, &descriptor, br#"{"score":1}"#, || false).unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(store.open_verified(&key, &descriptor).unwrap().is_none());
}

#[test]
fn cached_analysis_enforces_size_json_contract_and_canonical_form() {
    let cases: &[(&[u8], u64, Option<WorkflowErrorKind>)] = &[
        (br#"{"score":1}"#, 1, Some(WorkflowErrorKind::ResourceLimit)),
        (b"not-json", 1_024, Some(WorkflowErrorKind::Artifact)),
        (b"[]", 1_024, Some(WorkflowErrorKind::Artifact)),
        (
            br#"{ "score": 1 }"#,
            1_024,
            Some(WorkflowErrorKind::Artifact),
        ),
        (br#"{"score":1}"#, 1_024, None),
    ];
    for (bytes, limit, expected) in cases {
        let temp = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(temp.path().join("store"));
        let descriptor = request(ContentDigest::sha256(b"source"))
            .descriptor()
            .unwrap();
        let record = store.put(&descriptor, bytes).unwrap();
        let artifact = store
            .open_verified(&record.key, &descriptor)
            .unwrap()
            .unwrap();
        let result = cache::validate(&artifact, *limit, &mut || true);
        assert_eq!(result.as_ref().err().map(|error| error.kind), *expected);
    }
}

fn request(source_identity: ContentDigest) -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity,
        producer: ProducerFingerprint {
            name: "analysis-test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
        spec: MediaArtifactSpec::Analysis(AnalysisSpec {
            analysis_type: "scenes".into(),
            configuration: json!({}),
        }),
    }
}

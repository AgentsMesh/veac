use veac_artifact::*;

use super::*;

#[test]
fn expired_analysis_deadline_fails_before_source_or_cache_access() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let error = super::store(
        &store,
        &temp.path().join("missing"),
        &request(ContentDigest::sha256(b"source")),
        MediaArtifactLimits::default(),
        Instant::now(),
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(!store.root().exists());
}

#[test]
fn cached_analysis_read_and_fresh_store_obey_their_guards() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let request = request(ContentDigest::sha256(b"source"));
    let descriptor = request.descriptor().unwrap();
    let bytes = request.result.canonical_bytes(4_096).unwrap();
    let record = store.put(&descriptor, &bytes).unwrap();
    let artifact = store
        .open_verified(&record.key, &descriptor)
        .unwrap()
        .unwrap();
    assert_eq!(
        cache::validate(&artifact, 4_096, &mut || false)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ResourceLimit
    );

    let other = ArtifactStore::new(temp.path().join("other"));
    let key = artifact_key(&descriptor).unwrap();
    assert_eq!(
        store_fresh(&other, &descriptor, &bytes, || false)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ResourceLimit
    );
    assert!(other.open_verified(&key, &descriptor).unwrap().is_none());
}

#[test]
fn cached_analysis_enforces_size_type_digest_and_canonical_form() {
    let request = request(ContentDigest::sha256(b"source"));
    let canonical = request.result.canonical_bytes(4_096).unwrap();
    let mut noncanonical = canonical.clone();
    noncanonical.insert(1, b' ');
    let cases: Vec<(Vec<u8>, u64, Option<WorkflowErrorKind>)> = vec![
        (canonical.clone(), 1, Some(WorkflowErrorKind::ResourceLimit)),
        (
            b"not-json".to_vec(),
            4_096,
            Some(WorkflowErrorKind::Artifact),
        ),
        (b"[]".to_vec(), 4_096, Some(WorkflowErrorKind::Artifact)),
        (noncanonical, 4_096, Some(WorkflowErrorKind::Artifact)),
        (canonical, 4_096, None),
    ];
    for (bytes, limit, expected) in cases {
        let temp = tempfile::tempdir().unwrap();
        let store = ArtifactStore::new(temp.path().join("store"));
        let descriptor = request.descriptor().unwrap();
        let record = store.put(&descriptor, &bytes).unwrap();
        let artifact = store
            .open_verified(&record.key, &descriptor)
            .unwrap()
            .unwrap();
        let result = cache::validate(&artifact, limit, &mut || true);
        assert_eq!(result.as_ref().err().map(|error| error.kind), expected);
    }

    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("wrong-type"));
    let wrong = MediaArtifactRequest {
        source_identity: request.source_identity.clone(),
        producer: request.producer.clone(),
        spec: MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: veac_ir::StreamSelection {
                global_index: 0,
                type_index: 0,
            },
            source_clock: SourceClockSpec::Identity {
                duration: veac_ir::RationalTime::new(1, 1).unwrap(),
            },
            sample_rate: 48_000,
            channels: 2,
        }),
    }
    .descriptor()
    .unwrap();
    let canonical = request.result.canonical_bytes(4_096).unwrap();
    let record = store.put(&wrong, &canonical).unwrap();
    let artifact = store.open_verified(&record.key, &wrong).unwrap().unwrap();
    assert_eq!(
        cache::validate(&artifact, 4_096, &mut || true)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::Artifact
    );
}

fn request(source_identity: ContentDigest) -> AnalysisIngestionRequest {
    AnalysisIngestionRequest {
        source_identity,
        producer: ProducerFingerprint {
            name: "analysis-test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
        result: AnalysisResultEnvelope::new(
            AnalysisDescriptor::SceneBoundaries(SceneBoundaryAnalysisDescriptor {
                sensitivity_millionths: 500_000,
            }),
            AnalysisResult::SceneBoundaries(SceneBoundaryAnalysisResult {
                boundaries: vec![SceneBoundary {
                    at: veac_ir::RationalTime::new(1, 10).unwrap(),
                    confidence_millionths: 900_000,
                }],
            }),
        )
        .unwrap(),
    }
}

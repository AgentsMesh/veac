use std::fs;

use veac_artifact::{
    artifact_key, ArtifactStore, ContentDigest, MediaArtifactRequest, MediaArtifactSpec,
    ProducerFingerprint, ProxyAudioSpec, SourceClockSpec,
};
use veac_ir::{RationalTime, StreamSelection};

use super::*;

#[test]
fn postflight_proof_rejects_replacement_before_reverify_and_store() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("rendered.wav");
    fs::write(&path, b"runtime-verified-output").unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    let proof = OutputProof::capture(&path, 1_024, deadline).unwrap();
    fs::write(&path, b"replacement-after-postflight").unwrap();
    assert!(proof.reverify(&path, 1_024, deadline).is_err());

    let descriptor = request().descriptor().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    assert!(proof.store(&store, &descriptor, &path, deadline).is_err());
    assert!(store
        .get(&artifact_key(&descriptor).unwrap())
        .unwrap()
        .is_none());
}

#[test]
fn expired_deadline_prevents_fresh_output_publication() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("rendered.wav");
    fs::write(&path, b"runtime-verified-output").unwrap();
    let capture_deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    let proof = OutputProof::capture(&path, 1_024, capture_deadline).unwrap();
    let descriptor = request().descriptor().unwrap();
    let key = artifact_key(&descriptor).unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));

    let error = proof
        .store(&store, &descriptor, &path, std::time::Instant::now())
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(store.get(&key).unwrap().is_none());
}

fn request() -> MediaArtifactRequest {
    MediaArtifactRequest {
        source_identity: ContentDigest::sha256(b"source"),
        producer: ProducerFingerprint {
            name: "test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
        spec: MediaArtifactSpec::ProxyAudio(ProxyAudioSpec {
            source_stream: StreamSelection {
                global_index: 0,
                type_index: 0,
            },
            source_clock: SourceClockSpec::Identity {
                duration: RationalTime::new(1, 1).unwrap(),
            },
            sample_rate: 48_000,
            channels: 2,
        }),
    }
}

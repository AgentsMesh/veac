use std::time::Duration;
use veac_artifact::*;
use veac_ir::{RationalTime, StreamSelection};

use super::*;

#[test]
fn expired_deadline_stops_cached_payload_verification() {
    let temp = tempfile::tempdir().unwrap();
    let request = MediaArtifactRequest {
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
    };
    let descriptor = request.descriptor().unwrap();
    let key = artifact_key(&descriptor).unwrap();
    let empty = ArtifactStore::new(temp.path().join("empty"));
    assert!(load(
        &empty,
        &key,
        &descriptor,
        &SystemFfprobe::new("unused"),
        &request.spec,
        MediaArtifactLimits::default(),
        Instant::now() + Duration::from_secs(1),
    )
    .unwrap()
    .is_none());
    let store = ArtifactStore::new(temp.path().join("store"));
    store.put(&descriptor, b"cached").unwrap();

    let error = load(
        &store,
        &key,
        &descriptor,
        &SystemFfprobe::new("unused"),
        &request.spec,
        MediaArtifactLimits::default(),
        Instant::now(),
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(store.open_verified(&key, &descriptor).unwrap().is_some());
}

#[test]
fn cache_error_helpers_preserve_failure_kinds() {
    let io = ArtifactError::from(std::io::Error::other("cache I/O failed"));
    assert_eq!(cache_error(io).kind, WorkflowErrorKind::Artifact);
    assert_eq!(
        limit::<()>("cache limit reached").unwrap_err().kind,
        WorkflowErrorKind::ResourceLimit
    );
}

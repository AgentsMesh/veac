use veac_artifact::{ArtifactStore, ContentDigest};

use super::super::{runner, WorkflowErrorKind};
use super::support::fixture;

#[test]
fn commit_guard_cleans_staging_without_publishing_an_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let mut fixture = fixture();
    let payload = vec![0x5a; 256 * 1024];
    let path = temp.path().join("provider.payload");
    std::fs::write(&path, &payload).unwrap();
    let veac_provider::ProviderOutput::TextToSpeech(result) = &mut fixture.response.output else {
        unreachable!()
    };
    result.audio.record.content = ContentDigest::sha256(&payload);
    result.audio.record.size_bytes = payload.len() as u64;
    let artifact = &result.audio;
    let store = ArtifactStore::new(temp.path().join("store"));
    let target = store
        .root()
        .join("sha256")
        .join(&artifact.record.key.value[..2])
        .join(&artifact.record.key.value[2..]);

    let error = runner::commit_while(&store, artifact, &path, || !target.exists()).unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(!target.exists());
    assert!(store
        .open_verified(&artifact.record.key, &artifact.descriptor)
        .unwrap()
        .is_none());
}

use super::{verified, ComputationRecord, RecordOutput, RECORD_VERSION};
use crate::{ContentDigest, NodeCacheKey};
use veac_artifact::{
    ArtifactDescriptor, ArtifactParameters, ArtifactStore, ProducedArtifactParameters,
    ProducerFingerprint, RenderOutputParameters,
};

fn key() -> NodeCacheKey {
    NodeCacheKey::computation("record-test", 1, ContentDigest::sha256(b"record")).unwrap()
}

#[test]
fn record_rejects_wrong_identity_before_opening_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    for record in [
        ComputationRecord {
            version: RECORD_VERSION + 1,
            key: key().digest().value.clone(),
            outputs: Vec::new(),
        },
        ComputationRecord {
            version: RECORD_VERSION,
            key: ContentDigest::sha256(b"wrong").value,
            outputs: Vec::new(),
        },
    ] {
        assert!(record
            .open(&key(), &store)
            .unwrap_err()
            .message()
            .contains("identity"));
    }
}

#[test]
fn record_rejects_unsorted_outputs_before_artifact_lookup() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    let descriptor = ArtifactDescriptor::new(
        ProducerFingerprint {
            name: "record-test".to_owned(),
            version: "1".to_owned(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
        Vec::new(),
        ArtifactParameters::VideoMaster(ProducedArtifactParameters::Render(
            RenderOutputParameters::new(0, "output"),
        )),
    );
    let artifact = store.put(&descriptor, b"payload").unwrap().key;
    let record = ComputationRecord {
        version: RECORD_VERSION,
        key: key().digest().value.clone(),
        outputs: vec![
            RecordOutput {
                name: "z".to_owned(),
                artifact: artifact.clone(),
            },
            RecordOutput {
                name: "a".to_owned(),
                artifact,
            },
        ],
    };
    let error = record.open(&key(), &store).unwrap_err();
    assert!(error.message().contains("sorted"));
}

#[test]
fn record_maps_artifact_store_failures() {
    let temp = tempfile::tempdir().unwrap();
    let store_root = temp.path().join("store-file");
    std::fs::write(&store_root, b"x").unwrap();
    let error = verified(
        &ArtifactStore::new(store_root),
        &ContentDigest::sha256(b"missing"),
    )
    .unwrap_err();
    assert!(error.message().contains("artifact cache failure"));
}

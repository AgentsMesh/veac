use super::{checked_parent, checked_relative, checked_root, io_error, publish};
use crate::{BuildErrorKind, CancellationToken, ContentDigest};
use veac_artifact::{
    ArtifactDescriptor, ArtifactParameters, ArtifactStore, ProducedArtifactParameters,
    ProducerFingerprint, RenderOutputParameters,
};
use veac_project::DeliveryKind;

#[test]
fn delivery_paths_reject_non_directory_and_non_relative_contracts() {
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("file");
    std::fs::write(&file, b"x").unwrap();
    assert_eq!(
        checked_root(&file).unwrap_err().kind(),
        BuildErrorKind::InvalidContract
    );
    assert_eq!(
        checked_root(file.join("child")).unwrap_err().kind(),
        BuildErrorKind::Cache
    );

    for value in ["", "../escape", "/absolute", "nested/../escape"] {
        assert_eq!(
            checked_relative(value).unwrap_err().kind(),
            BuildErrorKind::InvalidContract
        );
    }

    let root = checked_root(temp.path().join("delivery")).unwrap();
    assert!(checked_parent(&root, std::path::Path::new("nested/deep"))
        .unwrap()
        .ends_with("nested/deep"));
    std::fs::write(root.join("blocked"), b"x").unwrap();
    assert_eq!(
        checked_parent(&root, std::path::Path::new("blocked/child"))
            .unwrap_err()
            .kind(),
        BuildErrorKind::InvalidContract
    );
    assert!(io_error(std::io::Error::other("disk"))
        .message()
        .contains("delivery filesystem failure"));
}

#[test]
fn file_delivery_reports_missing_artifacts_and_cancellation() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("artifacts"));
    let root = checked_root(temp.path().join("delivery")).unwrap();
    let missing = ContentDigest::sha256(b"missing");
    assert_eq!(
        publish(
            &store,
            &missing,
            &root,
            "missing.bin",
            DeliveryKind::File,
            &CancellationToken::new(),
        )
        .unwrap_err()
        .kind(),
        BuildErrorKind::Cache
    );

    let descriptor = ArtifactDescriptor::new(
        ProducerFingerprint {
            name: "delivery-test".to_owned(),
            version: "1".to_owned(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
        Vec::new(),
        ArtifactParameters::VideoMaster(ProducedArtifactParameters::Render(
            RenderOutputParameters::new(0, "output"),
        )),
    );
    let record = store.put(&descriptor, b"payload").unwrap();
    let token = CancellationToken::new();
    token.cancel();
    let error = publish(
        &store,
        &record.key,
        &root,
        "cancelled.bin",
        DeliveryKind::File,
        &token,
    )
    .unwrap_err();
    assert_eq!(error.kind(), BuildErrorKind::Cancelled);
}

#[test]
fn file_delivery_maps_broken_artifact_store_errors() {
    let temp = tempfile::tempdir().unwrap();
    let store_root = temp.path().join("store-file");
    std::fs::write(&store_root, b"x").unwrap();
    let root = checked_root(temp.path().join("delivery")).unwrap();
    let error = publish(
        &ArtifactStore::new(store_root),
        &ContentDigest::sha256(b"missing"),
        &root,
        "value.bin",
        DeliveryKind::File,
        &CancellationToken::new(),
    )
    .unwrap_err();
    assert!(error.message().contains("artifact delivery failure"));
}

use super::*;
use veac_artifact::{ArtifactDescriptor, ArtifactKind, ContentDigest, ProducerFingerprint};

#[test]
fn expired_inspect_is_typed_and_writes_no_output() {
    let temp = tempfile::tempdir().unwrap();
    let store_path = temp.path().join("store");
    let key = stored(&store_path);
    let output = temp.path().join("inspection.json");
    let error = inspect(&store_path, &key.value, Some(&output), Instant::now()).unwrap_err();
    assert!(error.is_resource_limit());
    assert!(!output.exists());
}

#[test]
fn expired_remove_is_typed_and_preserves_the_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let store_path = temp.path().join("store");
    let key = stored(&store_path);
    let error = remove(&store_path, &key.value, Instant::now()).unwrap_err();
    assert!(error.is_resource_limit());
    assert!(ArtifactStore::new(store_path).get(&key).unwrap().is_some());
}

#[test]
fn expired_materialize_is_typed_and_leaves_no_destination() {
    let temp = tempfile::tempdir().unwrap();
    let store_path = temp.path().join("store");
    let key = stored(&store_path);
    let destination = temp.path().join("materialized.bin");
    let error = materialize(&store_path, &key.value, &destination, Instant::now()).unwrap_err();
    assert!(error.is_resource_limit());
    assert!(!destination.exists());
}

#[test]
fn inspect_stream_verifies_payloads_larger_than_the_memory_api_limit() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("large.bin");
    let size = veac_artifact::MAX_IN_MEMORY_ARTIFACT_BYTES + 1;
    std::fs::File::create(&source)
        .unwrap()
        .set_len(size)
        .unwrap();
    let store_path = temp.path().join("store");
    let key = ArtifactStore::new(&store_path)
        .put_file(&descriptor(), &source)
        .unwrap()
        .key;
    let output = temp.path().join("inspection.json");
    inspect(
        &store_path,
        &key.value,
        Some(&output),
        Instant::now() + Duration::from_secs(veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS),
    )
    .unwrap();
    let value: serde_json::Value = serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(value["record"]["size_bytes"], size);
}

#[test]
fn command_dispatch_inspects_materializes_and_removes_an_artifact() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    let key = stored(&store);
    let inspection = temp.path().join("inspection.json");
    run(ArtifactCommand::Inspect {
        store: store.clone(),
        key: key.value.clone(),
        output: Some(inspection.clone()),
    })
    .unwrap();
    let value: serde_json::Value =
        serde_json::from_slice(&std::fs::read(inspection).unwrap()).unwrap();
    assert_eq!(value["record"]["key"]["value"], key.value);

    let destination = temp.path().join("payload.bin");
    run(ArtifactCommand::Materialize {
        store: store.clone(),
        key: key.value.clone(),
        destination: destination.clone(),
    })
    .unwrap();
    assert_eq!(std::fs::read(destination).unwrap(), b"payload");

    run(ArtifactCommand::Remove {
        store: store.clone(),
        key: key.value.clone(),
    })
    .unwrap();
    assert!(ArtifactStore::new(store).get(&key).unwrap().is_none());
}

#[test]
fn commands_report_absent_artifacts_and_invalid_keys() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    std::fs::create_dir(&store).unwrap();
    let absent = ContentDigest::sha256(b"absent").value;
    let inspect = run(ArtifactCommand::Inspect {
        store: store.clone(),
        key: absent.clone(),
        output: Some(temp.path().join("inspection.json")),
    })
    .unwrap_err();
    assert!(inspect.to_string().contains("ARTIFACT_NOT_FOUND"));
    let materialize = run(ArtifactCommand::Materialize {
        store: store.clone(),
        key: absent.clone(),
        destination: temp.path().join("payload.bin"),
    })
    .unwrap_err();
    assert!(materialize.to_string().contains("ARTIFACT_NOT_FOUND"));
    let remove = run(ArtifactCommand::Remove {
        store: store.clone(),
        key: absent,
    })
    .unwrap_err();
    assert!(remove.to_string().contains("ARTIFACT_NOT_FOUND"));

    let invalid = run(ArtifactCommand::Remove {
        store,
        key: "not-a-sha256-digest".to_owned(),
    })
    .unwrap_err();
    assert!(invalid.to_string().contains("ARTIFACT_FAILED"));
    assert!(!invalid.is_resource_limit());
}

#[test]
fn artifact_outputs_cannot_target_the_store() {
    let temp = tempfile::tempdir().unwrap();
    let store = temp.path().join("store");
    let key = stored(&store);
    let error = run(ArtifactCommand::Inspect {
        store: store.clone(),
        key: key.value.clone(),
        output: Some(store.join("inspection.json")),
    })
    .unwrap_err();
    assert!(error.to_string().contains("ARTIFACT_OUTPUT_CONFLICT"));

    let error = run(ArtifactCommand::Materialize {
        store: store.clone(),
        key: key.value,
        destination: store.join("payload.bin"),
    })
    .unwrap_err();
    assert!(error.to_string().contains("ARTIFACT_OUTPUT_CONFLICT"));
    reject_store_output(&store, None).unwrap();
}

#[test]
fn artifact_output_io_failures_are_typed() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("output.json");
    let error = reject_store_output(&temp.path().join("missing-store"), Some(&output)).unwrap_err();
    assert!(error.to_string().contains("ARTIFACT_FAILED"));

    let store = temp.path().join("store");
    let key = stored(&store);
    let missing_output = temp.path().join("missing-parent/output.json");
    let error = reject_store_output(&store, Some(&missing_output)).unwrap_err();
    assert!(error.to_string().contains("ARTIFACT_FAILED"));

    let directory = temp.path().join("directory");
    std::fs::create_dir(&directory).unwrap();
    let error = materialize(
        &store,
        &key.value,
        &directory,
        Instant::now() + Duration::from_secs(60),
    )
    .unwrap_err();
    assert!(error.to_string().contains("ARTIFACT_MATERIALIZE_FAILED"));
}

fn stored(path: &Path) -> ContentDigest {
    ArtifactStore::new(path)
        .put(&descriptor(), b"payload")
        .unwrap()
        .key
}

fn descriptor() -> ArtifactDescriptor {
    ArtifactDescriptor::new(
        ArtifactKind::Analysis,
        ProducerFingerprint {
            name: "fixture".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
        vec![],
        serde_json::json!({}),
    )
}

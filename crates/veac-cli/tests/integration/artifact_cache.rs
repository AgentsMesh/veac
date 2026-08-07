use assert_cmd::Command;
use predicates::prelude::*;
use veac_artifact::{
    ArtifactDescriptor, ArtifactParameters, ArtifactStore, ContentDigest, ProducerFingerprint,
    ProviderResultParameters,
};

#[test]
fn artifact_cli_inspects_verified_metadata_and_removes_by_key() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let descriptor = ArtifactDescriptor::new(
        ProducerFingerprint {
            name: "test-analysis".to_owned(),
            version: "1".to_owned(),
            configuration: ContentDigest::sha256(b"config"),
        },
        vec![],
        ArtifactParameters::Speech(ProviderResultParameters::new("speech").unwrap()),
    );
    let payload = br#"{"scenes":[]}"#;
    let record = store.put(&descriptor, payload).unwrap();
    let output = temp.path().join("inspection.json");
    let conflicting = store.root().join("inspection.json");
    let materialized = temp.path().join("materialized.json");
    veac()
        .args([
            "artifact",
            "materialize",
            store.root().to_str().unwrap(),
            &record.key.value,
            materialized.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Materialized artifact"));
    assert_eq!(std::fs::read(materialized).unwrap(), payload);

    veac()
        .args([
            "artifact",
            "materialize",
            store.root().to_str().unwrap(),
            &record.key.value,
            conflicting.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ARTIFACT_OUTPUT_CONFLICT"));

    veac()
        .args([
            "artifact",
            "inspect",
            store.root().to_str().unwrap(),
            &record.key.value,
            "--output",
            output.to_str().unwrap(),
        ])
        .assert()
        .success();
    let inspected: serde_json::Value =
        serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(
        inspected["record"]["content"],
        serde_json::to_value(&record.content).unwrap()
    );

    veac()
        .args([
            "artifact",
            "inspect",
            store.root().to_str().unwrap(),
            &record.key.value,
            "--output",
            conflicting.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ARTIFACT_OUTPUT_CONFLICT"));

    veac()
        .args([
            "artifact",
            "remove",
            store.root().to_str().unwrap(),
            &record.key.value,
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("Removed artifact"));
    assert!(store.get(&record.key).unwrap().is_none());
}

#[test]
fn artifact_cli_rejects_invalid_and_missing_keys() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store");
    veac()
        .args(["artifact", "inspect", root.to_str().unwrap(), "bad"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ARTIFACT_FAILED"));
    veac()
        .args([
            "artifact",
            "remove",
            root.to_str().unwrap(),
            &ContentDigest::sha256(b"missing").value,
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("ARTIFACT_NOT_FOUND"));
}

fn veac() -> Command {
    Command::cargo_bin("veac").unwrap()
}

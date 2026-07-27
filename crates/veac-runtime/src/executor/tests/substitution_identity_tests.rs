use veac_artifact::{ArtifactStore, ContentDigest, DigestAlgorithm};

use super::support::*;
use crate::executor::BundleExecutor;

#[test]
fn substitution_proof_changes_checkpoint_identity_and_resume_behavior() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "output.bin");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let first_bundle = bundle(vec![video_task("master", &output)]);
    let first = executor.execute_runtime(&first_bundle, &store).unwrap();

    let mut second_bundle = first_bundle.clone();
    second_bundle.substitution_proof = ContentDigest::sha256(b"different-substitution");
    let second = executor.execute_runtime(&second_bundle, &store).unwrap();
    let resumed = executor.execute_runtime(&second_bundle, &store).unwrap();
    assert!(!first.tasks[0].cache_hit);
    assert!(!second.tasks[0].cache_hit);
    assert!(resumed.tasks[0].cache_hit);
    assert_ne!(
        first.tasks[0].checkpoint.key,
        second.tasks[0].checkpoint.key
    );
    assert_eq!(executor.environment().calls.borrow().len(), 2);
}

#[test]
fn plan_identity_changes_checkpoint_identity_without_an_external_hash() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "output.bin");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let first_bundle = bundle(vec![video_task("master", &output)]);
    let first = executor.execute_runtime(&first_bundle, &store).unwrap();

    let mut second_bundle = first_bundle.clone();
    second_bundle.plan_identity = ContentDigest::sha256(b"different-plan");
    let second = executor.execute_runtime(&second_bundle, &store).unwrap();
    let resumed = executor.execute_runtime(&second_bundle, &store).unwrap();
    assert!(!second.tasks[0].cache_hit);
    assert!(resumed.tasks[0].cache_hit);
    assert_ne!(
        first.tasks[0].checkpoint.key,
        second.tasks[0].checkpoint.key
    );
    assert_eq!(executor.environment().calls.borrow().len(), 2);
}

#[test]
fn malformed_substitution_proof_fails_before_tool_fingerprinting() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "output.bin");
    let mut value = bundle(vec![video_task("master", &output)]);
    value.substitution_proof = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: "bad".into(),
    };
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(&value, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap_err();
    assert!(error.message.contains("substitution proof"));
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
    assert_eq!(executor.environment().calls.borrow().len(), 0);
}

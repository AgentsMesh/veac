use veac_artifact::{ArtifactStore, ContentDigest};

use super::support::*;
use crate::executor::BundleExecutor;

#[test]
fn write_file_is_atomic_and_resumes_from_a_verified_checkpoint() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "captions.srt");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let environment = FakeFfmpeg::default();
    let executor = BundleExecutor::new(environment);
    let bundle = bundle(vec![write_task("captions", &output, b"caption\n")]);

    let first = executor.execute_runtime(&bundle, &store).unwrap();
    assert!(!first.tasks[0].cache_hit);
    assert_eq!(first.tasks[0].outputs, vec![output.clone()]);
    assert_eq!(first.tasks[0].output_records.len(), 1);
    assert_eq!(
        first.tasks[0].output_records[0].content,
        ContentDigest::sha256(b"caption\n")
    );
    assert_eq!(first.tasks[0].output_records[0].size_bytes, 8);
    first.tasks[0].output_records[0].key.validate().unwrap();
    assert_eq!(std::fs::read(&output).unwrap(), b"caption\n");
    let second = executor.execute_runtime(&bundle, &store).unwrap();
    assert!(second.tasks[0].cache_hit);
    assert_eq!(second.tasks[0].outputs, first.tasks[0].outputs);
    assert_eq!(
        second.tasks[0].output_records,
        first.tasks[0].output_records
    );
    assert!(executor.environment().calls.borrow().is_empty());
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
}

#[test]
fn changed_output_content_forces_a_rerender() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.bin");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let bundle = bundle(vec![video_task("master", &output)]);

    executor.execute_runtime(&bundle, &store).unwrap();
    std::fs::write(&output, b"corrupt").unwrap();
    let rerun = executor.execute_runtime(&bundle, &store).unwrap();
    assert!(!rerun.tasks[0].cache_hit);
    assert_eq!(executor.environment().calls.borrow().len(), 2);
    assert_eq!(std::fs::read(output).unwrap(), b"rendered-output");
}

#[test]
fn empty_caption_sidecar_is_a_valid_atomic_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "empty.srt");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let bundle = bundle(vec![write_task("empty", &output, b"")]);
    executor.execute_runtime(&bundle, &store).unwrap();
    assert_eq!(std::fs::metadata(&output).unwrap().len(), 0);
    assert!(executor.execute_runtime(&bundle, &store).unwrap().tasks[0].cache_hit);
}

#[cfg(unix)]
#[test]
fn commit_rejects_a_replaced_target_and_invalidates_its_checkpoint() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.bin");
    let victim = path(temp.path(), "victim.bin");
    let store_root = path(temp.path(), "store");
    std::fs::write(&victim, b"stable").unwrap();
    let replacement = output.clone();
    let mut executor = BundleExecutor::new(FakeFfmpeg::default());
    executor.checkpoint_stored_observer = Box::new(move |deadline| {
        symlink(&victim, &replacement).unwrap();
        deadline
    });

    let error = executor
        .execute_runtime(
            &bundle(vec![video_task("master", &output)]),
            &ArtifactStore::new(&store_root),
        )
        .unwrap_err();

    assert!(error.message.contains("unsafe output"));
    assert_no_checkpoint_payload(&store_root);
}

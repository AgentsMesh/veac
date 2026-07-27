use std::cell::Cell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use veac_artifact::ArtifactStore;

use super::support::*;
use crate::executor::{BundleExecutor, BundleSetupLimits, TaskExecutionLimits};
use crate::RuntimeErrorKind;

#[test]
fn tiny_setup_deadline_stops_resource_hash_before_any_publication() {
    let temp = tempfile::tempdir().unwrap();
    let input = path(temp.path(), "large-input.bin");
    let output = path(temp.path(), "captions.srt");
    let store_root = path(temp.path(), "store");
    std::fs::write(&input, vec![7_u8; 16 * 1024 * 1024]).unwrap();
    let mut value = bundle(vec![write_task("captions", &output, b"caption")]);
    value.protected_resources.push(protected_resource(&input));
    let executor = BundleExecutor::new(FakeFfmpeg::default())
        .with_setup_limits(BundleSetupLimits::new(Duration::from_micros(100)).unwrap());

    let error = executor
        .execute_runtime(&value, &ArtifactStore::new(&store_root))
        .unwrap_err();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
    assert!(!output.exists());
    assert_no_checkpoint_payload(&store_root);
}

#[test]
fn tightened_task_deadline_allows_a_verified_resume_hit() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "captions.srt");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default())
        .with_limits(TaskExecutionLimits::new(Duration::from_secs(30)).unwrap());
    let value = bundle(vec![write_task("captions", &output, b"caption")]);

    assert!(!executor.execute_runtime(&value, &store).unwrap().tasks[0].cache_hit);
    assert!(executor.execute_runtime(&value, &store).unwrap().tasks[0].cache_hit);
}

#[test]
fn task_expiry_after_checkpoint_store_publishes_neither_checkpoint_nor_output() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "captions.srt");
    let store_root = path(temp.path(), "store");
    let observed = Rc::new(Cell::new(false));
    let observer = Rc::clone(&observed);
    let mut executor = BundleExecutor::new(FakeFfmpeg::default())
        .with_limits(TaskExecutionLimits::new(Duration::from_secs(30)).unwrap());
    executor.checkpoint_stored_observer = Box::new(move |_| {
        observer.set(true);
        Instant::now()
    });

    let error = executor
        .execute_runtime(
            &bundle(vec![write_task("captions", &output, b"caption")]),
            &ArtifactStore::new(&store_root),
        )
        .unwrap_err();

    assert!(observed.get());
    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
    assert!(!output.exists());
    assert_no_checkpoint_payload(&store_root);
}

fn assert_no_checkpoint_payload(root: &std::path::Path) {
    fn contains_payload(path: &std::path::Path) -> bool {
        std::fs::read_dir(path).is_ok_and(|entries| {
            entries.filter_map(Result::ok).any(|entry| {
                entry.file_name() == "payload.bin"
                    || (entry.path().is_dir() && contains_payload(&entry.path()))
            })
        })
    }
    assert!(!contains_payload(root));
}

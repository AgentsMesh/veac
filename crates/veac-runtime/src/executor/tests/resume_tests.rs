use veac_artifact::ArtifactStore;

use super::support::*;
use crate::executor::BundleExecutor;

#[test]
fn image_sequence_resume_validates_every_frame_and_removes_stale_frames() {
    let temp = tempfile::tempdir().unwrap();
    let pattern = path(temp.path(), "frame-%d.png");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let bundle = bundle(vec![image_task("frames", &pattern)]);

    let first = executor.execute_runtime(&bundle, &store).unwrap();
    assert_eq!(first.tasks[0].outputs.len(), 2);
    assert_eq!(executor.environment().calls.borrow().len(), 1);
    let hit = executor.execute_runtime(&bundle, &store).unwrap();
    assert!(hit.tasks[0].cache_hit);
    assert_eq!(executor.environment().calls.borrow().len(), 1);

    std::fs::write(path(temp.path(), "frame-2.png"), b"corrupt").unwrap();
    std::fs::write(path(temp.path(), "frame-99.png"), b"stale").unwrap();
    let rerun = executor.execute_runtime(&bundle, &store).unwrap();
    assert!(!rerun.tasks[0].cache_hit);
    assert_eq!(executor.environment().calls.borrow().len(), 2);
    assert!(!path(temp.path(), "frame-99.png").exists());
    assert_eq!(
        std::fs::read(path(temp.path(), "frame-2.png")).unwrap(),
        b"frame-two"
    );
}

#[test]
fn two_pass_resume_requires_verified_passlogs_and_tracks_dependency() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.mp4");
    let log = path(temp.path(), "master.mp4.veac-pass-0.log");
    let tree = path(temp.path(), "master.mp4.veac-pass-0.log.mbtree");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let bundle = bundle(two_pass_tasks("master", &output));

    let first = executor.execute_runtime(&bundle, &store).unwrap();
    assert_eq!(first.tasks.len(), 2);
    assert_eq!(first.tasks[0].outputs, vec![log.clone(), tree.clone()]);
    assert_eq!(executor.environment().calls.borrow().len(), 2);
    let hit = executor.execute_runtime(&bundle, &store).unwrap();
    assert!(hit.tasks.iter().all(|task| task.cache_hit));
    assert_eq!(executor.environment().calls.borrow().len(), 2);

    std::fs::write(&log, b"corrupt").unwrap();
    let resumed = executor.execute_runtime(&bundle, &store).unwrap();
    assert!(!resumed.tasks[0].cache_hit);
    assert!(resumed.tasks[1].cache_hit);
    assert_eq!(executor.environment().calls.borrow().len(), 3);
    assert_eq!(std::fs::read(log).unwrap(), b"passlog");
}

#[test]
fn task_content_is_part_of_the_checkpoint_identity() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "captions.srt");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    executor
        .execute_runtime(
            &bundle(vec![write_task("captions", &output, b"one")]),
            &store,
        )
        .unwrap();
    let changed = executor
        .execute_runtime(
            &bundle(vec![write_task("captions", &output, b"two")]),
            &store,
        )
        .unwrap();
    assert!(!changed.tasks[0].cache_hit);
    assert_eq!(std::fs::read(output).unwrap(), b"two");
}

#[test]
fn corrupt_checkpoint_cache_is_discarded_and_rebuilt() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.bin");
    let root = path(temp.path(), "store");
    let store = ArtifactStore::new(&root);
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let bundle = bundle(vec![video_task("master", &output)]);
    let first = executor.execute_runtime(&bundle, &store).unwrap();
    let key = &first.tasks[0].checkpoint.key.value;
    let payload = root
        .join("sha256")
        .join(&key[..2])
        .join(&key[2..])
        .join("payload.bin");
    std::fs::write(payload, b"corrupt-checkpoint").unwrap();

    let rebuilt = executor.execute_runtime(&bundle, &store).unwrap();
    assert!(!rebuilt.tasks[0].cache_hit);
    assert_eq!(executor.environment().calls.borrow().len(), 2);
}

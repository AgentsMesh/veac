use veac_artifact::ArtifactStore;

use super::support::*;
use crate::executor::BundleExecutor;

#[test]
fn publication_adopts_same_named_sources_from_multiple_tasks() {
    let temp = tempfile::tempdir().unwrap();
    let first = path(temp.path(), "first.bin");
    let second = path(temp.path(), "second.bin");
    let executor = BundleExecutor::new(FakeFfmpeg::default());

    executor
        .execute_runtime(
            &bundle(vec![
                video_task("first", &first),
                video_task("second", &second),
            ]),
            &ArtifactStore::new(path(temp.path(), "store")),
        )
        .unwrap();

    assert_eq!(std::fs::read(first).unwrap(), b"rendered-output");
    assert_eq!(std::fs::read(second).unwrap(), b"rendered-output");
    assert_no_staging(temp.path());
}

#[test]
fn final_publication_reenumerates_late_image_sequence_members() {
    let temp = tempfile::tempdir().unwrap();
    let pattern = path(temp.path(), "frame-%d.png");
    let master = path(temp.path(), "master.bin");
    let late = path(temp.path(), "frame-99.png");
    let environment = FakeFfmpeg {
        mutate_on_call: Some((2, late.clone(), b"late-stale".to_vec())),
        ..FakeFfmpeg::default()
    };
    let executor = BundleExecutor::new(environment);

    executor
        .execute_runtime(
            &bundle(vec![
                image_task("frames", &pattern),
                video_task("master", &master),
            ]),
            &ArtifactStore::new(path(temp.path(), "store")),
        )
        .unwrap();

    assert!(!late.exists());
    assert_eq!(
        std::fs::read(path(temp.path(), "frame-1.png")).unwrap(),
        b"frame-one"
    );
    assert_eq!(std::fs::read(master).unwrap(), b"rendered-output");
}

#[test]
fn final_publication_reenumerates_late_passlog_members() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.mp4");
    let late = path(temp.path(), "master.mp4.veac-pass-9.log");
    let environment = FakeFfmpeg {
        mutate_on_call: Some((2, late.clone(), b"late-stale".to_vec())),
        ..FakeFfmpeg::default()
    };
    let executor = BundleExecutor::new(environment);

    executor
        .execute_runtime(
            &bundle(two_pass_tasks("master", &output)),
            &ArtifactStore::new(path(temp.path(), "store")),
        )
        .unwrap();

    assert!(!late.exists());
    assert_eq!(std::fs::read(output).unwrap(), b"rendered-output");
}

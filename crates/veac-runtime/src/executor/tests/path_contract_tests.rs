use veac_artifact::ArtifactStore;
use veac_codegen::emitter::{BackendAction, BackendInput};

use super::support::*;
use crate::executor::BundleExecutor;

#[test]
fn output_may_not_alias_a_protected_ffmpeg_input() {
    let temp = tempfile::tempdir().unwrap();
    let input = path(temp.path(), "source.bin");
    std::fs::write(&input, b"protected-source").unwrap();
    let mut task = video_task("master", &input);
    command_mut(&mut task).inputs.push(BackendInput {
        path: input.clone(),
    });
    let mut bundle = bundle(vec![task]);
    bundle.protected_resources.push(protected_resource(&input));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(&bundle, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap_err();
    assert!(error.message.contains("aliases a protected input"));
    assert_eq!(std::fs::read(input).unwrap(), b"protected-source");
    assert!(executor.environment().calls.borrow().is_empty());
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
}

#[test]
fn every_ffmpeg_input_must_be_declared_as_protected() {
    let temp = tempfile::tempdir().unwrap();
    let input = path(temp.path(), "source.bin");
    let output = path(temp.path(), "master.bin");
    std::fs::write(&input, b"source").unwrap();
    let mut task = video_task("master", &output);
    command_mut(&mut task)
        .inputs
        .push(BackendInput { path: input });
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(
            &bundle(vec![task]),
            &ArtifactStore::new(path(temp.path(), "store")),
        )
        .unwrap_err();
    assert!(error
        .message
        .contains("must be a protected bundle resource"));
    assert!(!output.exists());
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
}

#[test]
fn image_pattern_may_not_consume_a_protected_resource() {
    let temp = tempfile::tempdir().unwrap();
    let input = path(temp.path(), "frame-1.png");
    let pattern = path(temp.path(), "frame-%d.png");
    std::fs::write(&input, b"protected-frame").unwrap();
    let mut task = image_task("frames", &pattern);
    command_mut(&mut task).inputs.push(BackendInput {
        path: input.clone(),
    });
    let mut bundle = bundle(vec![task]);
    bundle.protected_resources.push(protected_resource(&input));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(&bundle, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap_err();
    assert!(error.message.contains("aliases a protected input"));
    assert_eq!(std::fs::read(input).unwrap(), b"protected-frame");
}

#[test]
fn static_and_intersecting_pattern_outputs_are_rejected_atomically() {
    let temp = tempfile::tempdir().unwrap();
    let static_path = path(temp.path(), "frame-1.png");
    let pattern = path(temp.path(), "frame-%d.png");
    let nested = path(temp.path(), "frame-1%d.png");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    for bundle in [
        bundle(vec![
            image_task("frames", &pattern),
            video_task("still", &static_path),
        ]),
        bundle(vec![
            image_task("frames", &pattern),
            image_task("nested", &nested),
        ]),
    ] {
        let error = executor.execute_runtime(&bundle, &store).unwrap_err();
        assert!(error.message.contains("image patterns overlap"));
    }
    assert!(executor.environment().calls.borrow().is_empty());
    assert!(!static_path.exists());
}

#[test]
fn outputs_may_not_enter_the_artifact_store_directory() {
    let temp = tempfile::tempdir().unwrap();
    let store_root = path(temp.path(), "store");
    std::fs::create_dir(&store_root).unwrap();
    let output = path(&store_root, "master.bin");
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(
            &bundle(vec![video_task("master", &output)]),
            &ArtifactStore::new(&store_root),
        )
        .unwrap_err();
    assert!(error
        .message
        .contains("artifact store management directory"));
    assert!(executor.environment().calls.borrow().is_empty());
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
}

#[test]
fn existing_unsafe_static_and_pattern_targets_fail_contract_preflight() {
    let temp = tempfile::tempdir().unwrap();
    let static_directory = path(temp.path(), "master.bin");
    let pattern_directory = path(temp.path(), "frame-1.png");
    std::fs::create_dir(&static_directory).unwrap();
    std::fs::create_dir(&pattern_directory).unwrap();
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let store = ArtifactStore::new(path(temp.path(), "store"));

    let static_error = executor
        .execute_runtime(
            &bundle(vec![video_task("master", &static_directory)]),
            &store,
        )
        .unwrap_err();
    assert!(static_error.message.contains("regular non-symlink file"));
    let pattern_error = executor
        .execute_runtime(
            &bundle(vec![image_task(
                "frames",
                &path(temp.path(), "frame-%d.png"),
            )]),
            &store,
        )
        .unwrap_err();
    assert!(pattern_error.message.contains("not a regular file"));
    assert!(executor.environment().calls.borrow().is_empty());
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
}

fn command_mut(
    task: &mut veac_codegen::emitter::BackendTask,
) -> &mut veac_codegen::emitter::BackendCommand {
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        panic!()
    };
    command
}

use veac_artifact::ArtifactStore;
use veac_codegen::emitter::{BackendAction, BackendInput, BackendTask};

use super::support::*;
use crate::executor::BundleExecutor;

#[test]
fn changed_resource_fails_before_fingerprint_or_checkpoint_resume() {
    let temp = tempfile::tempdir().unwrap();
    let input = path(temp.path(), "input.bin");
    let output = path(temp.path(), "output.bin");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    std::fs::write(&input, b"version-one").unwrap();
    let value = resource_bundle(video_task("master", &output), &input);
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    executor.execute_runtime(&value, &store).unwrap();

    std::fs::write(&input, b"version-two").unwrap();
    let error = executor.execute_runtime(&value, &store).unwrap_err();
    assert!(error.message.contains("resource identity changed"));
    assert_eq!(executor.environment().fingerprint_calls.get(), 1);
    assert_eq!(executor.environment().calls.borrow().len(), 1);
    assert_eq!(std::fs::read(output).unwrap(), b"rendered-output");
}

#[test]
fn resource_identity_is_part_of_the_checkpoint_fingerprint() {
    let temp = tempfile::tempdir().unwrap();
    let input = path(temp.path(), "input.bin");
    let output = path(temp.path(), "output.bin");
    let store = ArtifactStore::new(path(temp.path(), "store"));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    std::fs::write(&input, b"version-one").unwrap();
    let first = executor
        .execute_runtime(
            &resource_bundle(video_task("master", &output), &input),
            &store,
        )
        .unwrap();

    std::fs::write(&input, b"version-two").unwrap();
    let second = executor
        .execute_runtime(
            &resource_bundle(video_task("master", &output), &input),
            &store,
        )
        .unwrap();
    assert!(!second.tasks[0].cache_hit);
    assert_ne!(
        first.tasks[0].checkpoint.key,
        second.tasks[0].checkpoint.key
    );
    assert_eq!(executor.environment().calls.borrow().len(), 2);
}

#[test]
fn action_time_mutation_discards_staging_without_a_checkpoint() {
    let temp = tempfile::tempdir().unwrap();
    let input = path(temp.path(), "input.bin");
    let output = path(temp.path(), "master.mp4");
    let store_root = path(temp.path(), "store");
    std::fs::write(&input, b"stable-input").unwrap();
    let mut tasks = two_pass_tasks("master", &output);
    for task in &mut tasks {
        add_input(task, &input);
    }
    let mut value = bundle(tasks);
    value.protected_resources.push(protected_resource(&input));
    let executor = BundleExecutor::new(FakeFfmpeg {
        mutate_on_call: Some((1, input.clone(), b"changed-during-action".to_vec())),
        ..FakeFfmpeg::default()
    });
    let error = executor
        .execute_runtime(&value, &ArtifactStore::new(&store_root))
        .unwrap_err();
    assert!(error.message.contains("resource identity changed"));
    assert_eq!(executor.environment().calls.borrow().len(), 1);
    assert_eq!(executor.environment().fingerprint_calls.get(), 1);
    assert!(!output.exists());
    assert!(!path(temp.path(), "master.mp4.veac-pass-0.log").exists());
    assert!(!store_root.exists());
    assert_no_staging(temp.path());
}

#[test]
fn post_checkpoint_mutation_invalidates_checkpoint_before_commit() {
    let temp = tempfile::tempdir().unwrap();
    let input = path(temp.path(), "input.bin");
    let output = path(temp.path(), "output.bin");
    let store_root = path(temp.path(), "store");
    std::fs::write(&input, b"stable-input").unwrap();
    let value = resource_bundle(video_task("master", &output), &input);
    let changed = input.clone();
    let mut executor = BundleExecutor::new(FakeFfmpeg::default());
    executor.checkpoint_stored_observer = Box::new(move |deadline| {
        std::fs::write(&changed, b"changed").unwrap();
        deadline
    });
    let error = executor
        .execute_runtime(&value, &ArtifactStore::new(&store_root))
        .unwrap_err();
    assert!(error.message.contains("resource identity changed"));
    assert!(!output.exists());
    assert_no_checkpoint_payload(&store_root);
    assert_no_staging(temp.path());
}

#[test]
fn non_ascii_resource_order_contract_uses_utf8_strings() {
    let temp = tempfile::tempdir().unwrap();
    let first = path(temp.path(), "\u{e000}-resource.bin");
    let second = path(temp.path(), "\u{10000}-resource.bin");
    std::fs::write(&first, b"first").unwrap();
    std::fs::write(&second, b"second").unwrap();
    let output = path(temp.path(), "captions.srt");
    let mut value = bundle(vec![write_task("captions", &output, b"caption")]);
    value.protected_resources = vec![protected_resource(&second), protected_resource(&first)];
    value.protected_resources.sort_by(|left, right| {
        left.path
            .to_str()
            .unwrap()
            .cmp(right.path.to_str().unwrap())
    });
    BundleExecutor::new(FakeFfmpeg::default())
        .execute_runtime(&value, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap();
    assert_eq!(std::fs::read(output).unwrap(), b"caption");
}

fn resource_bundle(
    mut task: BackendTask,
    input: &std::path::Path,
) -> crate::executor::model::RuntimeBundle {
    add_input(&mut task, input);
    let mut value = bundle(vec![task]);
    value.protected_resources.push(protected_resource(input));
    value
}

fn add_input(task: &mut BackendTask, input: &std::path::Path) {
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        panic!("test task must use FFmpeg")
    };
    command.inputs.push(BackendInput {
        path: input.to_path_buf(),
    });
}

fn assert_no_staging(parent: &std::path::Path) {
    assert!(!std::fs::read_dir(parent).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".veac-stage-")
    }));
}

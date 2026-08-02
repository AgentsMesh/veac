use veac_artifact::ArtifactStore;
use veac_codegen::emitter::{BackendAction, BackendTask};

use super::support::*;
use crate::executor::BundleExecutor;

#[test]
fn passlog_sidecars_may_not_alias_protected_or_declared_outputs() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.mp4");
    let sidecar = path(temp.path(), "master.mp4.veac-pass-0.log.mbtree");
    std::fs::write(&sidecar, b"protected").unwrap();

    let mut protected = bundle(two_pass_tasks("master", &output));
    protected
        .protected_resources
        .push(protected_resource(&sidecar));
    assert_rejected_without_execution(&protected, temp.path(), "aliases a protected input");
    assert_eq!(std::fs::read(&sidecar).unwrap(), b"protected");

    let mut tasks = two_pass_tasks("master", &output);
    tasks.push(write_task("captions", &sidecar, b"replacement"));
    assert_rejected_without_execution(
        &bundle(tasks),
        temp.path(),
        "backend output paths or image patterns overlap",
    );
    assert_eq!(std::fs::read(sidecar).unwrap(), b"protected");

    let mut tasks = two_pass_tasks("master", &output);
    tasks.push(image_task(
        "frames",
        &path(temp.path(), "master.mp4.veac-pass-%d.log.mbtree"),
    ));
    assert_rejected_without_execution(
        &bundle(tasks),
        temp.path(),
        "backend output paths or image patterns overlap",
    );
}

#[test]
fn malformed_pass_numbers_and_prefixes_fail_contract_preflight() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.mp4");

    let mut wrong_number = two_pass_tasks("number", &output);
    replace_option(&mut wrong_number[1], "-pass", "1");
    assert_rejected_without_execution(
        &bundle(wrong_number),
        temp.path(),
        "invalid FFmpeg pass number",
    );

    let mut wrong_prefix = two_pass_tasks("prefix", &output);
    replace_option(
        &mut wrong_prefix[1],
        "-passlogfile",
        path(temp.path(), "other-pass").to_str().unwrap(),
    );
    assert_rejected_without_execution(
        &bundle(wrong_prefix),
        temp.path(),
        "same FFmpeg passlog prefix",
    );

    let mut single = video_task("single", &output);
    command_mut(&mut single)
        .output_args
        .extend(["-pass".to_owned(), "1".to_owned()]);
    assert_rejected_without_execution(
        &bundle(vec![single]),
        temp.path(),
        "single-pass task may not declare FFmpeg pass state",
    );
    assert!(!output.exists());
}

#[test]
fn image_pattern_may_not_consume_the_artifact_store_or_its_ancestor() {
    let temp = tempfile::tempdir().unwrap();
    let pattern = path(temp.path(), "frame-%d.png");
    for root in [
        path(temp.path(), "frame-1.png"),
        path(temp.path(), "frame-2.png/checkpoints"),
    ] {
        let executor = BundleExecutor::new(FakeFfmpeg::default());
        let error = executor
            .execute_runtime(
                &bundle(vec![image_task("frames", &pattern)]),
                &ArtifactStore::new(&root),
            )
            .unwrap_err();
        assert!(error
            .message
            .contains("artifact store management directory"));
        assert!(executor.environment().calls.borrow().is_empty());
        assert_eq!(executor.environment().fingerprint_calls.get(), 0);
        assert!(!root.exists());
    }
}

#[test]
fn two_pass_cleanup_preserves_non_passlog_prefix_files() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.mp4");
    let unrelated = path(temp.path(), "master.mp4.veac-pass-not-a-log");
    std::fs::write(&unrelated, b"keep").unwrap();
    BundleExecutor::new(FakeFfmpeg::default())
        .execute_runtime(
            &bundle(two_pass_tasks("master", &output)),
            &ArtifactStore::new(path(temp.path(), "store")),
        )
        .unwrap();
    assert_eq!(std::fs::read(unrelated).unwrap(), b"keep");
}

fn assert_rejected_without_execution(
    value: &crate::executor::model::RuntimeBundle,
    parent: &std::path::Path,
    message: &str,
) {
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(value, &ArtifactStore::new(path(parent, "store")))
        .unwrap_err();
    assert!(error.message.contains(message), "{}", error.message);
    assert!(executor.environment().calls.borrow().is_empty());
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
}

fn replace_option(task: &mut BackendTask, name: &str, value: &str) {
    let command = command_mut(task);
    let index = command
        .output_args
        .iter()
        .position(|candidate| candidate == name)
        .unwrap();
    command.output_args[index + 1] = value.to_owned();
}

fn command_mut(task: &mut BackendTask) -> &mut veac_codegen::emitter::BackendCommand {
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        panic!("test task must use FFmpeg")
    };
    command
}

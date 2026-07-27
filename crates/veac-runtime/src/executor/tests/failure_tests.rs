use veac_artifact::ArtifactStore;
use veac_codegen::emitter::BackendRequirement;

use super::support::*;
use crate::executor::BundleExecutor;

#[test]
fn unavailable_encoder_fails_before_any_task_starts() {
    let temp = tempfile::tempdir().unwrap();
    let caption = path(temp.path(), "captions.srt");
    let video = path(temp.path(), "master.bin");
    let mut bundle = bundle(vec![
        write_task("captions", &caption, b"caption"),
        video_task("master", &video),
    ]);
    bundle.requirements.push(BackendRequirement::Encoder {
        deliverable_id: deliverable_id("master"),
        name: "dnxhd".to_owned(),
    });
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(&bundle, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap_err();
    assert!(error.message.contains("encoder dnxhd"));
    assert!(!caption.exists());
    assert!(!video.exists());
    assert!(executor.environment().calls.borrow().is_empty());
    assert_eq!(executor.environment().fingerprint_calls.get(), 1);
    assert_eq!(executor.environment().encoder_calls.get(), 1);
    assert_eq!(executor.environment().muxer_calls.get(), 0);
}

#[test]
fn generated_bundle_declares_software_encoder_before_fake_execution() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.mp4");
    let (bundle, _) = super::sealed_generated_bundle(output.clone());
    assert!(bundle.requirements().iter().any(|value| matches!(
        value,
        BackendRequirement::Encoder { name, .. } if name == "libx264"
    )));
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute(&bundle, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap_err();
    assert!(error.message.contains("encoder libx264"));
    assert!(!output.exists());
    assert!(executor.environment().calls.borrow().is_empty());
}

#[test]
fn failed_task_preserves_its_previous_output() {
    let temp = tempfile::tempdir().unwrap();
    let first = path(temp.path(), "first.bin");
    let second = path(temp.path(), "second.bin");
    std::fs::write(&second, b"previous").unwrap();
    let environment = FakeFfmpeg {
        fail_on_call: Some(2),
        ..FakeFfmpeg::default()
    };
    let executor = BundleExecutor::new(environment);
    let error = executor
        .execute_runtime(
            &bundle(vec![
                video_task("first", &first),
                video_task("second", &second),
            ]),
            &ArtifactStore::new(path(temp.path(), "store")),
        )
        .unwrap_err();
    assert!(error.message.contains("fake FFmpeg failure"));
    assert_eq!(std::fs::read(first).unwrap(), b"rendered-output");
    assert_eq!(std::fs::read(second).unwrap(), b"previous");
}

#[test]
fn invalid_contract_fails_before_fingerprinting_or_execution() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.bin");
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let mut value = bundle(vec![video_task("master", &output)]);
    value.plan_identity.value = "not-a-digest".to_owned();
    let error = executor
        .execute_runtime(&value, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap_err();
    assert!(error.message.contains("invalid render plan hash"));
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
    assert!(executor.environment().calls.borrow().is_empty());
}

#[cfg(unix)]
#[test]
fn existing_static_symlink_fails_contract_preflight() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let victim = path(temp.path(), "victim.txt");
    let output = path(temp.path(), "captions.srt");
    std::fs::write(&victim, b"do-not-touch").unwrap();
    symlink(&victim, &output).unwrap();
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(
            &bundle(vec![write_task("captions", &output, b"replacement")]),
            &ArtifactStore::new(path(temp.path(), "store")),
        )
        .unwrap_err();
    assert!(error.message.contains("regular non-symlink file"));
    assert_eq!(std::fs::read(victim).unwrap(), b"do-not-touch");
    assert!(std::fs::symlink_metadata(output)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(executor.environment().calls.borrow().is_empty());
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
}

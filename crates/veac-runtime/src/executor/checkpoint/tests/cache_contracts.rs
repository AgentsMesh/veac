use std::time::{Duration, Instant};
use veac_artifact::ArtifactStore;
use veac_codegen::emitter::BackendProduct;

use super::super::{
    invalidate,
    manifest::{self, CheckpointManifest, CheckpointOutput},
    resume, store, validate_cached, StagedFile,
};
use super::support::{deadline, digest, entry, identity_for, write_task};
use crate::RuntimeErrorKind;

#[test]
fn expired_resume_validate_and_invalidate_are_resource_limits() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("caption.srt");
    let task = write_task(&output, b"caption");
    let identity = identity_for(&task);
    let store_path = ArtifactStore::new(temp.path().join("store"));

    let resume_error = match resume(&store_path, &task, &identity, Instant::now()) {
        Err(error) => error,
        Ok(_) => panic!("expired checkpoint resume must fail"),
    };
    let validation_error = validate_cached(&task, &identity, b"{}", Instant::now()).unwrap_err();
    let invalidation_error = invalidate(&store_path, &identity.key, Instant::now()).unwrap_err();

    for error in [resume_error, validation_error, invalidation_error] {
        assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
    }
}

#[test]
fn fresh_output_hash_expiry_does_not_publish_a_checkpoint() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("caption.srt");
    let staged = temp.path().join("large-staged.srt");
    std::fs::write(&staged, vec![9_u8; 16 * 1024 * 1024]).unwrap();
    let task = write_task(&output, b"caption");
    let identity = identity_for(&task);
    let root = temp.path().join("store");
    let store_path = ArtifactStore::new(&root);

    let error = store(
        &store_path,
        &task,
        &identity,
        &[StagedFile {
            source: staged,
            target: output,
            allow_empty: true,
        }],
        Instant::now() + Duration::from_micros(100),
    )
    .unwrap_err();

    assert_eq!(error.kind, RuntimeErrorKind::ResourceLimit);
    assert!(!contains_payload(&root));
}

#[test]
fn checkpoint_store_and_resume_report_real_store_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("store-file");
    std::fs::write(&root, b"not a directory").unwrap();
    let store_path = ArtifactStore::new(&root);
    let task = write_task(&temp.path().join("caption.srt"), b"caption");
    let identity = identity_for(&task);
    let error = match resume(&store_path, &task, &identity, deadline()) {
        Err(error) => error,
        Ok(_) => panic!("file-backed store must fail"),
    };
    assert!(error
        .message
        .contains("cannot invalidate render checkpoint"));
    let error = invalidate(&store_path, &identity.key, deadline()).unwrap_err();
    assert!(error
        .message
        .contains("cannot invalidate render checkpoint"));

    let staging = temp.path().join("staged.srt");
    std::fs::write(&staging, b"caption").unwrap();
    let error = store(
        &store_path,
        &task,
        &identity,
        &[StagedFile {
            source: staging,
            target: temp.path().join("caption.srt"),
            allow_empty: true,
        }],
        deadline(),
    )
    .unwrap_err();
    assert!(error.message.contains("cannot store render checkpoint"));
}

#[test]
fn changed_descriptor_and_missing_outputs_invalidate_cached_entries() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("caption.srt");
    let task = write_task(&output, b"caption");
    let identity = identity_for(&task);
    let store_path = ArtifactStore::new(temp.path().join("store"));
    let staged = temp.path().join("staged.srt");
    std::fs::write(&staged, b"caption").unwrap();
    store(
        &store_path,
        &task,
        &identity,
        &[StagedFile {
            source: staged,
            target: output.clone(),
            allow_empty: true,
        }],
        deadline(),
    )
    .unwrap();

    let changed_task = write_task(&output, b"different");
    let changed_identity = identity_for(&changed_task);
    assert!(
        resume(&store_path, &changed_task, &changed_identity, deadline())
            .unwrap()
            .is_none()
    );
    assert!(resume(&store_path, &task, &identity, deadline())
        .unwrap()
        .is_none());
    assert_eq!(
        identity.descriptor.kind(),
        veac_artifact::ArtifactKind::RenderCheckpoint
    );
    assert_eq!(task.product, BackendProduct::CaptionSidecar);
    assert_eq!(identity.descriptor.dependencies[0].identity, digest(1));
}

#[test]
fn cached_output_path_and_descriptor_tampering_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("caption.srt");
    std::fs::write(&output, b"caption").unwrap();
    let task = write_task(&output, b"caption");
    let identity = identity_for(&task);
    let path = output.to_string_lossy().into_owned();

    let mut wrong_path = entry(&path);
    match &mut wrong_path {
        CheckpointOutput::File { path, .. } => path.push_str(".other"),
        CheckpointOutput::Package { .. } => panic!("fixture is a file checkpoint"),
    }
    let payload = manifest::encode(&CheckpointManifest {
        schema_version: 2,
        outputs: vec![wrong_path],
    })
    .unwrap();
    assert!(validate_cached(&task, &identity, &payload, deadline())
        .unwrap_err()
        .message
        .contains("path changed"));

    let mut wrong_descriptor = entry(&path);
    let CheckpointOutput::File { descriptor, .. } = &mut wrong_descriptor else {
        panic!("fixture is a file checkpoint");
    };
    let veac_artifact::ArtifactParameters::CaptionSidecar(parameters) = &mut descriptor.parameters
    else {
        panic!("fixture is a caption descriptor");
    };
    parameters.index = 1;
    let payload = manifest::encode(&CheckpointManifest {
        schema_version: 2,
        outputs: vec![wrong_descriptor],
    })
    .unwrap();
    assert!(validate_cached(&task, &identity, &payload, deadline())
        .unwrap_err()
        .message
        .contains("descriptor changed"));
}

fn contains_payload(path: &std::path::Path) -> bool {
    std::fs::read_dir(path).is_ok_and(|entries| {
        entries.filter_map(Result::ok).any(|entry| {
            entry.file_name() == "payload.bin"
                || (entry.path().is_dir() && contains_payload(&entry.path()))
        })
    })
}

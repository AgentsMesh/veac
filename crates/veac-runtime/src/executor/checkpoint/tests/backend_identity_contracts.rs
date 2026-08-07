use std::path::Path;

use veac_codegen::emitter::{BackendOutput, BackendProduct};

use super::support::{digest, fingerprint, task, write_task};

#[test]
fn ffmpeg_configuration_changes_the_checkpoint_key() {
    let task = task(
        BackendOutput::File("master.mp4".into()),
        BackendProduct::VideoMaster,
    );
    let first = fingerprint();
    let mut second = first.clone();
    second.configuration = digest(8);
    let identity = |fingerprint| {
        super::super::identity(&task, &digest(1), &digest(2), Some(fingerprint), None).unwrap()
    };
    let first_identity = identity(&first);
    let second_identity = identity(&second);
    assert_ne!(first_identity.key, second_identity.key);
    assert_eq!(
        first_identity.descriptor.producer,
        crate::workflow::media_artifact_producer(&first).unwrap()
    );
}

#[test]
fn direct_write_checkpoint_binds_the_exact_backend_identity() {
    let identity = super::super::identity(
        &write_task(Path::new("captions.srt"), b"caption"),
        &digest(1),
        &digest(2),
        None,
        None,
    )
    .unwrap();
    assert_eq!(
        identity.descriptor.producer.configuration,
        crate::artifact_backend_identity()
    );
    assert_eq!(identity.descriptor.producer.name, "veac-runtime");
}

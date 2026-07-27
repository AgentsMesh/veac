use std::path::PathBuf;

use veac_artifact::{ArtifactKind, ContentDigest, DigestAlgorithm};
use veac_codegen::emitter::{
    BackendAction, BackendFilterBinding, BackendFilterContract, BackendFilterEscape, BackendOutput,
    BackendProduct,
};

use super::super::identity;
use super::support::{digest, fingerprint, identity_for, task, write_task};

#[test]
fn task_identity_requires_its_producer_and_valid_dependencies() {
    let task = task(
        BackendOutput::File(PathBuf::from("master.mp4")),
        BackendProduct::VideoMaster,
    );
    let missing = match super::super::identity(&task, &digest(1), &digest(2), None, None) {
        Err(error) => error,
        Ok(_) => panic!("FFmpeg task without fingerprint must fail"),
    };
    assert!(missing.message.contains("missing its producer fingerprint"));

    let invalid = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: "invalid".to_owned(),
    };
    let error = match super::super::identity(
        &write_task(PathBuf::from("caption.srt").as_path(), b"caption"),
        &invalid,
        &digest(2),
        None,
        None,
    ) {
        Err(error) => error,
        Ok(_) => panic!("invalid plan dependency must fail"),
    };
    assert!(error.message.contains("cannot identify render checkpoint"));
}

#[test]
fn task_identity_hashes_every_action_and_output_shape() {
    let write = identity_for(&write_task(
        PathBuf::from("caption.srt").as_path(),
        b"caption",
    ));
    let files = identity_for(&task(
        BackendOutput::Files {
            paths: vec![PathBuf::from("left.wav"), PathBuf::from("right.wav")],
        },
        BackendProduct::AudioStem,
    ));
    let sequence = identity_for(&task(
        BackendOutput::ImageSequence {
            pattern: PathBuf::from("frame-%d.png"),
        },
        BackendProduct::ImageSequence,
    ));
    assert_ne!(write.key, files.key);
    assert_ne!(files.key, sequence.key);

    let with_predecessor = super::super::identity(
        &task(
            BackendOutput::File(PathBuf::from("master.mp4")),
            BackendProduct::VideoMaster,
        ),
        &digest(1),
        &digest(2),
        Some(&fingerprint()),
        Some(&digest(4)),
    )
    .unwrap();
    assert_eq!(with_predecessor.descriptor.dependencies.len(), 3);
}

#[test]
fn directory_membership_is_part_of_the_task_identity() {
    let first = directory_task("font-a.ttf");
    let second = directory_task("font-b.ttf");
    let first_identity = identity_for(&first);
    let second_identity = identity_for(&second);
    assert_ne!(first_identity.key, second_identity.key);

    let (BackendAction::Ffmpeg(first_command), BackendAction::Ffmpeg(second_command)) =
        (&first.action, &second.action)
    else {
        unreachable!()
    };
    assert_eq!(first_command.to_args(), second_command.to_args());
}

#[test]
fn file_filter_binding_is_part_of_the_task_identity() {
    let first = identity_for(&file_task("grade-a.cube"));
    let second = identity_for(&file_task("grade-b.cube"));
    assert_ne!(first.key, second.key);
}

#[test]
fn output_descriptor_maps_every_backend_product_kind() {
    let identity = identity_for(&write_task(
        PathBuf::from("caption.srt").as_path(),
        b"caption",
    ));
    for (product, kind) in [
        (BackendProduct::VideoMaster, ArtifactKind::VideoMaster),
        (
            BackendProduct::RenderPassLog,
            ArtifactKind::RenderCheckpoint,
        ),
        (
            BackendProduct::ImageSequence,
            ArtifactKind::ImageSequenceFrame,
        ),
        (BackendProduct::CaptionSidecar, ArtifactKind::CaptionSidecar),
        (BackendProduct::AudioStem, ArtifactKind::AudioStem),
        (BackendProduct::VideoWaveform, ArtifactKind::VideoWaveform),
        (BackendProduct::Vectorscope, ArtifactKind::Vectorscope),
        (BackendProduct::Histogram, ArtifactKind::Histogram),
    ] {
        assert_eq!(identity::output(&identity, product, "output", 0).kind, kind);
    }
}

fn directory_task(file: &str) -> veac_codegen::emitter::BackendTask {
    const TOKEN: &str = "__VEAC_FILTER_RESOURCE_0000__";
    let mut task = task(
        BackendOutput::File(PathBuf::from("master.mp4")),
        BackendProduct::VideoMaster,
    );
    let contract = BackendFilterContract::new(
        TOKEN.to_owned(),
        vec![BackendFilterBinding::directory(
            TOKEN.to_owned(),
            PathBuf::from("fonts"),
            vec![PathBuf::from(file)],
            BackendFilterEscape::Quoted,
        )],
    )
    .unwrap();
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        unreachable!()
    };
    command.filter_graph = Some(contract.render_original().unwrap());
    command.filter_contract = Some(contract);
    task
}

fn file_task(file: &str) -> veac_codegen::emitter::BackendTask {
    const TOKEN: &str = "__VEAC_FILTER_RESOURCE_0000__";
    let mut task = task(
        BackendOutput::File(PathBuf::from("master.mp4")),
        BackendProduct::VideoMaster,
    );
    let contract = BackendFilterContract::new(
        TOKEN.to_owned(),
        vec![BackendFilterBinding::file(
            TOKEN.to_owned(),
            PathBuf::from(file),
            BackendFilterEscape::FilterValue,
        )],
    )
    .unwrap();
    let BackendAction::Ffmpeg(command) = &mut task.action else {
        unreachable!()
    };
    command.filter_graph = Some(contract.render_original().unwrap());
    command.filter_contract = Some(contract);
    task
}

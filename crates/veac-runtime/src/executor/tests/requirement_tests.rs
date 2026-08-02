use veac_artifact::ArtifactStore;
use veac_codegen::emitter::BackendRequirement;

use super::support::*;
use crate::executor::BundleExecutor;
use crate::RuntimeErrorKind;

#[test]
fn unavailable_mxf_muxer_fails_before_any_task_starts() {
    let temp = tempfile::tempdir().unwrap();
    let caption = path(temp.path(), "captions.ass");
    let video = path(temp.path(), "master.mxf");
    let mut value = bundle(vec![
        write_task("captions", &caption, b"caption"),
        video_task("master", &video),
    ]);
    value.requirements.push(BackendRequirement::Muxer {
        deliverable_id: deliverable_id("master"),
        name: "mxf".to_owned(),
    });
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(&value, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap_err();
    assert!(error.message.contains("muxer mxf"));
    assert!(!caption.exists());
    assert!(!video.exists());
    assert!(executor.environment().calls.borrow().is_empty());
    assert_eq!(executor.environment().encoder_calls.get(), 0);
    assert_eq!(executor.environment().muxer_calls.get(), 1);
}

#[test]
fn duplicate_and_orphaned_muxer_requirements_fail_bundle_contract() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.mxf");
    let requirement = BackendRequirement::Muxer {
        deliverable_id: deliverable_id("master"),
        name: "mxf".to_owned(),
    };
    let mut duplicate = bundle(vec![video_task("master", &output)]);
    duplicate.requirements = vec![requirement.clone(), requirement];
    assert_contract_error(&duplicate, temp.path());

    let mut orphan = bundle(vec![video_task("master", &output)]);
    orphan.requirements.push(BackendRequirement::Muxer {
        deliverable_id: deliverable_id("absent"),
        name: "mxf".to_owned(),
    });
    assert_contract_error(&orphan, temp.path());
}

#[test]
fn every_available_capability_kind_allows_execution_after_one_query_each() {
    let temp = tempfile::tempdir().unwrap();
    let output = path(temp.path(), "master.mxf");
    let mut value = bundle(vec![video_task("master", &output)]);
    value.requirements = vec![
        BackendRequirement::Encoder {
            deliverable_id: deliverable_id("master"),
            name: "dnxhd".to_owned(),
        },
        BackendRequirement::Muxer {
            deliverable_id: deliverable_id("master"),
            name: "mxf".to_owned(),
        },
        BackendRequirement::Decoder {
            deliverable_id: deliverable_id("master"),
            name: "h264".to_owned(),
        },
        BackendRequirement::Demuxer {
            deliverable_id: deliverable_id("master"),
            name: "mov".to_owned(),
        },
        BackendRequirement::Filter {
            deliverable_id: deliverable_id("master"),
            name: "zscale".to_owned(),
        },
        BackendRequirement::HardwareBackend {
            deliverable_id: deliverable_id("master"),
            name: "videotoolbox".to_owned(),
        },
        BackendRequirement::HardwareDevice {
            deliverable_id: deliverable_id("master"),
            name: "videotoolbox".to_owned(),
        },
    ];
    let environment = FakeFfmpeg {
        encoders: BTreeSet::from(["dnxhd".to_owned()]),
        muxers: BTreeSet::from(["mxf".to_owned()]),
        decoders: BTreeSet::from(["h264".to_owned()]),
        demuxers: BTreeSet::from(["mov".to_owned()]),
        filters: BTreeSet::from(["zscale".to_owned()]),
        hardware_backends: BTreeSet::from(["videotoolbox".to_owned()]),
        hardware_devices: BTreeSet::from(["videotoolbox".to_owned()]),
        ..FakeFfmpeg::default()
    };
    let executor = BundleExecutor::new(environment);
    executor
        .execute_runtime(&value, &ArtifactStore::new(path(temp.path(), "store")))
        .unwrap();
    assert!(output.is_file());
    assert_eq!(executor.environment().calls.borrow().len(), 1);
    assert_eq!(executor.environment().encoder_calls.get(), 1);
    assert_eq!(executor.environment().muxer_calls.get(), 1);
    assert_eq!(executor.environment().decoder_calls.get(), 1);
    assert_eq!(executor.environment().demuxer_calls.get(), 1);
    assert_eq!(executor.environment().filter_calls.get(), 1);
    assert_eq!(executor.environment().hardware_backend_calls.get(), 1);
    assert_eq!(executor.environment().hardware_device_calls.get(), 1);
}

#[test]
fn unavailable_typed_capability_kinds_fail_before_spawn() {
    let requirements = [
        BackendRequirement::Decoder {
            deliverable_id: deliverable_id("master"),
            name: "h264".to_owned(),
        },
        BackendRequirement::Demuxer {
            deliverable_id: deliverable_id("master"),
            name: "mov".to_owned(),
        },
        BackendRequirement::Filter {
            deliverable_id: deliverable_id("master"),
            name: "zscale".to_owned(),
        },
        BackendRequirement::HardwareBackend {
            deliverable_id: deliverable_id("master"),
            name: "videotoolbox".to_owned(),
        },
        BackendRequirement::HardwareDevice {
            deliverable_id: deliverable_id("master"),
            name: "videotoolbox".to_owned(),
        },
    ];
    for requirement in requirements {
        let temp = tempfile::tempdir().unwrap();
        let output = path(temp.path(), "master.bin");
        let mut value = bundle(vec![video_task("master", &output)]);
        let expected = requirement.kind().as_str();
        value.requirements.push(requirement);
        let executor = BundleExecutor::new(FakeFfmpeg::default());
        let error = executor
            .execute_runtime(&value, &ArtifactStore::new(path(temp.path(), "store")))
            .unwrap_err();
        assert_eq!(error.kind, RuntimeErrorKind::MissingBackendCapability);
        assert!(error.message.contains(expected), "{}", error.message);
        assert!(executor.environment().calls.borrow().is_empty());
        assert!(!output.exists());
    }
}

fn assert_contract_error(value: &crate::executor::model::RuntimeBundle, parent: &std::path::Path) {
    let executor = BundleExecutor::new(FakeFfmpeg::default());
    let error = executor
        .execute_runtime(value, &ArtifactStore::new(path(parent, "store")))
        .unwrap_err();
    assert!(error.message.contains("requirement"), "{}", error.message);
    assert_eq!(executor.environment().fingerprint_calls.get(), 0);
    assert!(executor.environment().calls.borrow().is_empty());
}
use std::collections::BTreeSet;

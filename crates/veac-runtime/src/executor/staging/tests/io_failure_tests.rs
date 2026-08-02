use std::path::PathBuf;

use veac_codegen::emitter::{
    BackendAction, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};

use super::super::common_parent_from_outputs;
use super::super::directory::{Directory, EntryState};
use super::super::write::stage as stage_write;
use super::deadline;

#[test]
fn descriptor_relative_directory_failures_keep_operation_context() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing");
    assert!(Directory::open(&missing)
        .unwrap_err()
        .message
        .contains("open directory"));

    let directory = Directory::open(temp.path()).unwrap();
    std::fs::create_dir(temp.path().join("existing")).unwrap();
    assert!(directory
        .create_child("existing")
        .unwrap_err()
        .message
        .contains("create transaction directory"));
    assert!(directory
        .child("absent")
        .unwrap_err()
        .message
        .contains("open transaction directory"));
    assert!(directory
        .state("invalid\0name")
        .unwrap_err()
        .message
        .contains("inspect transaction path"));
}

#[cfg(unix)]
#[test]
fn descriptor_relative_mutation_and_open_fail_closed_on_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let temp = tempfile::tempdir().unwrap();
    let removable = temp.path().join("removable");
    let unreadable = temp.path().join("unreadable");
    std::fs::write(&removable, b"value").unwrap();
    std::fs::write(&unreadable, b"value").unwrap();
    let directory = Directory::open(temp.path()).unwrap();
    let EntryState::Regular(removable_identity) = directory.state("removable").unwrap() else {
        panic!("fixture is regular");
    };
    let EntryState::Regular(unreadable_identity) = directory.state("unreadable").unwrap() else {
        panic!("fixture is regular");
    };

    std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o500)).unwrap();
    let remove = directory.remove_bound("removable", removable_identity);
    std::fs::set_permissions(temp.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    assert!(remove
        .unwrap_err()
        .message
        .contains("remove transaction path"));

    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o000)).unwrap();
    let sync = directory.sync_bound("unreadable", unreadable_identity);
    std::fs::set_permissions(&unreadable, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(sync
        .unwrap_err()
        .message
        .contains("open transaction output"));
}

#[test]
fn stage_write_and_parent_discovery_report_missing_storage() {
    assert!(common_parent_from_outputs(&[])
        .unwrap_err()
        .message
        .contains("no output parent"));

    let root = tempfile::tempdir().unwrap();
    let directory = tempfile::tempdir_in(root.path()).unwrap();
    let descriptor = Directory::open(directory.path()).unwrap();
    let target = root.path().join("output");
    let task = write_task(target);
    std::fs::remove_dir(directory.path()).unwrap();
    assert!(
        stage_write(&directory, &descriptor, &task, b"value", deadline())
            .unwrap_err()
            .message
            .contains("create transaction file")
    );
}

fn write_task(target: PathBuf) -> BackendTask {
    BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_io_failure").unwrap(),
        phase: BackendPhase::Single,
        product: BackendProduct::CaptionSidecar,
        output: BackendOutput::File(target.clone()),
        action: BackendAction::WriteFile {
            path: target,
            content: b"value".to_vec(),
        },
    }
}

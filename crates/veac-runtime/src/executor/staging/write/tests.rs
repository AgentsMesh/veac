use std::path::PathBuf;

use veac_codegen::emitter::{
    BackendAction, BackendOutput, BackendPhase, BackendProduct, BackendTask,
};

use super::{stage, write_error};
use crate::executor::staging::directory::Directory;

#[test]
fn write_stage_requires_a_single_file_output() {
    let directory = tempfile::tempdir().unwrap();
    let descriptor = Directory::open(directory.path()).unwrap();
    let task = BackendTask {
        deliverable_id: veac_ir::DeliverableId::new("dlv_write_contract").unwrap(),
        phase: BackendPhase::Single,
        product: BackendProduct::CaptionSidecar,
        output: BackendOutput::Files {
            paths: vec![PathBuf::from("caption.vtt")],
        },
        action: BackendAction::WriteFile {
            path: "caption.vtt".into(),
            content: b"caption".to_vec(),
        },
    };
    let error = stage(
        &directory,
        &descriptor,
        &task,
        b"caption",
        std::time::Instant::now() + std::time::Duration::from_secs(10),
    )
    .unwrap_err();
    assert!(error.message.contains("requires one file output"));
}

#[test]
fn write_errors_keep_staging_context() {
    let error = write_error(std::io::Error::other("fixture failure"));
    assert!(error.message.contains("cannot stage output file"));
    assert!(error.message.contains("fixture failure"));
}

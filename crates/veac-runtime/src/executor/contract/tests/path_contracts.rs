use std::collections::BTreeSet;

use veac_codegen::emitter::{BackendAction, BackendCommand, BackendInput, BackendOutput};
use veac_ir::DeliverableId;

use super::super::paths;
use super::support::bundle;

#[test]
fn missing_ffmpeg_inputs_preserve_backend_path_context() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing-input");
    let mut value = bundle(&temp.path().join("output.srt"));
    value.tasks[0].action = BackendAction::Ffmpeg(BackendCommand {
        preparations: vec![],
        inputs: vec![BackendInput { path: missing }],
        filter_graph: None,
        filter_contract: None,
        maps: Vec::new(),
        output_args: Vec::new(),
        output_path: temp.path().join("output.srt"),
    });

    let error = paths::validate(&value, &BTreeSet::new()).unwrap_err();

    assert!(error
        .message
        .contains("cannot validate protected backend path"));
}

#[test]
fn bundle_outputs_must_share_one_publication_directory() {
    let temp = tempfile::tempdir().unwrap();
    let first = temp.path().join("first").join("one.srt");
    let second = temp.path().join("second").join("two.srt");
    std::fs::create_dir_all(first.parent().unwrap()).unwrap();
    std::fs::create_dir_all(second.parent().unwrap()).unwrap();
    let mut value = bundle(&first);
    let mut task = value.tasks[0].clone();
    task.deliverable_id = DeliverableId::new("dlv_second").unwrap();
    task.output = BackendOutput::File(second.clone());
    task.action = BackendAction::WriteFile {
        path: second,
        content: b"second".to_vec(),
    };
    value.tasks.push(task);

    let error = paths::validate(&value, &BTreeSet::new()).unwrap_err();

    assert!(error.message.contains("one output directory"));
}

#[cfg(unix)]
#[test]
fn backend_paths_must_be_utf8() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let path = std::path::PathBuf::from(OsString::from_vec(vec![0xff]));
    assert!(paths::utf8(&path)
        .unwrap_err()
        .message
        .contains("valid UTF-8"));
}

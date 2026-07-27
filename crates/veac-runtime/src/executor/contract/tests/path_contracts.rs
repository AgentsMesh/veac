use std::collections::BTreeSet;

use veac_codegen::emitter::{BackendAction, BackendCommand, BackendInput};

use super::super::paths;
use super::support::bundle;

#[test]
fn missing_ffmpeg_inputs_preserve_backend_path_context() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing-input");
    let mut value = bundle(&temp.path().join("output.srt"));
    value.tasks[0].action = BackendAction::Ffmpeg(BackendCommand {
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

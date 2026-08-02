use std::path::Path;

use veac_codegen::emitter::{BackendAction, BackendOutput, BackendPhase};

use super::super::{current_paths, ensure_safe_parent, passlog_prefix};
use super::support::{appended, task};

#[test]
fn current_paths_sorts_file_lists_and_requires_the_declared_passlog() {
    let temp = tempfile::tempdir().unwrap();
    let a = temp.path().join("a.wav");
    let b = temp.path().join("b.wav");
    let files = task(
        BackendOutput::Files {
            paths: vec![b.clone(), a.clone()],
        },
        BackendPhase::Single,
    );
    assert_eq!(current_paths(&files).unwrap(), vec![a, b]);

    let prefix = temp.path().join("master.pass");
    let expected = appended(&prefix, "-0.log");
    let mut first = task(
        BackendOutput::File(expected.clone()),
        BackendPhase::FirstPass,
    );
    let BackendAction::Ffmpeg(command) = &mut first.action else {
        unreachable!()
    };
    command.output_args = vec![
        "-passlogfile".to_owned(),
        prefix.to_string_lossy().into_owned(),
    ];
    assert!(current_paths(&first).unwrap().is_empty());
    std::fs::write(&expected, b"log").unwrap();
    std::fs::write(appended(&prefix, "-0.log.mbtree"), b"tree").unwrap();
    assert_eq!(current_paths(&first).unwrap().len(), 2);
}

#[test]
fn passlog_prefix_requires_one_ffmpeg_argument() {
    let temp = tempfile::tempdir().unwrap();
    let output = temp.path().join("pass.log");
    let mut write = task(BackendOutput::File(output.clone()), BackendPhase::FirstPass);
    write.action = BackendAction::WriteFile {
        path: output.clone(),
        content: Vec::new(),
    };
    assert!(passlog_prefix(&write)
        .unwrap_err()
        .message
        .contains("must be an FFmpeg action"));

    let mut ffmpeg = task(BackendOutput::File(output), BackendPhase::FirstPass);
    let BackendAction::Ffmpeg(command) = &mut ffmpeg.action else {
        unreachable!()
    };
    command.output_args.clear();
    assert!(passlog_prefix(&ffmpeg)
        .unwrap_err()
        .message
        .contains("requires one"));
    let BackendAction::Ffmpeg(command) = &mut ffmpeg.action else {
        unreachable!()
    };
    command.output_args = vec![
        "-passlogfile".into(),
        "first".into(),
        "-passlogfile".into(),
        "second".into(),
    ];
    assert!(passlog_prefix(&ffmpeg)
        .unwrap_err()
        .message
        .contains("requires one"));
}

#[test]
fn safe_parent_reports_traversal_missing_file_and_root_paths() {
    let temp = tempfile::tempdir().unwrap();
    assert!(ensure_safe_parent(Path::new("parent/../output"))
        .unwrap_err()
        .message
        .contains("may not contain '..'"));
    assert!(ensure_safe_parent(&temp.path().join("missing/output"))
        .unwrap_err()
        .message
        .contains("unavailable"));
    let file = temp.path().join("not-a-directory");
    std::fs::write(&file, b"value").unwrap();
    assert!(ensure_safe_parent(&file.join("output"))
        .unwrap_err()
        .message
        .contains("non-symlink directory"));
    assert!(ensure_safe_parent(Path::new("/"))
        .unwrap_err()
        .message
        .contains("no file name"));
    assert_eq!(
        ensure_safe_parent(Path::new("output.bin")).unwrap(),
        Path::new(".")
    );
}

#[cfg(unix)]
#[test]
fn safe_parent_rejects_a_symlink_directory() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let real = temp.path().join("real");
    let link = temp.path().join("link");
    std::fs::create_dir(&real).unwrap();
    symlink(real, &link).unwrap();
    assert!(ensure_safe_parent(&link.join("output"))
        .unwrap_err()
        .message
        .contains("non-symlink directory"));
}

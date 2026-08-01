use veac_codegen::emitter::BackendOutput;

use super::super::StagedFile;
use super::{apply, task};

#[test]
fn empty_and_escaped_transactions_are_rejected_before_prepare() {
    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let error = apply(staging.path(), &[], &[], false).unwrap_err();
    assert!(error.message.contains("contains no output files"));

    let outside = temp.path().join("outside-source");
    std::fs::write(&outside, b"untrusted").unwrap();
    let file = StagedFile {
        source: outside.clone(),
        target: temp.path().join("output"),
        allow_empty: false,
    };
    let error = apply(staging.path(), &[file], &[], false).unwrap_err();
    assert!(error.message.contains("escaped its transaction directory"));
    assert_eq!(std::fs::read(outside).unwrap(), b"untrusted");
    assert!(!temp.path().join("output").exists());
}

#[cfg(unix)]
#[test]
fn transaction_rejects_a_non_utf8_source_name() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;

    let temp = tempfile::tempdir().unwrap();
    let staging = tempfile::tempdir_in(temp.path()).unwrap();
    let source = staging.path().join(OsString::from_vec(vec![0xff]));
    let file = StagedFile {
        source,
        target: temp.path().join("output"),
        allow_empty: false,
    };

    let error = apply(staging.path(), &[file], &[], false).unwrap_err();

    assert!(error.message.contains("file name must be valid UTF-8"));
}

#[test]
fn one_atomic_task_cannot_span_output_directories() {
    let temp = tempfile::tempdir().unwrap();
    let left = temp.path().join("left");
    let right = temp.path().join("right");
    std::fs::create_dir(&left).unwrap();
    std::fs::create_dir(&right).unwrap();
    let output = BackendOutput::Files {
        paths: vec![left.join("a.out"), right.join("b.out")],
    };

    let error = super::super::common_parent(&output).unwrap_err();

    assert!(error.message.contains("may not span output directories"));
}

#[test]
fn failed_prepared_task_discards_its_unjournaled_staging_area() {
    let root = tempfile::tempdir().unwrap();
    let directory = tempfile::Builder::new()
        .prefix(".veac-stage-")
        .tempdir_in(root.path())
        .unwrap();
    let staging = directory.path().to_owned();
    let staged_task = task(
        directory,
        vec![StagedFile {
            source: staging.join("missing"),
            target: root.path().join("output"),
            allow_empty: false,
        }],
    );
    let parent = std::fs::canonicalize(root.path()).unwrap();
    let locks = crate::executor::locking::acquire_until(&[parent], super::deadline()).unwrap();

    let error = staged_task.commit(&locks, super::deadline()).unwrap_err();

    assert!(error.message.contains("staged output is missing"));
    assert!(!staging.exists());
}

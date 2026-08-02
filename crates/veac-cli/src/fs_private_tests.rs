use std::io;
use std::path::Path;

use super::{atomic_write, atomic_write_impl, io_error};

#[test]
fn write_io_errors_retain_the_operation_and_path() {
    let error = io_error(
        "write temporary output",
        Path::new("result.json"),
        io::Error::other("disk unavailable"),
    );

    let rendered = error.to_string();
    assert!(rendered.contains("WRITE_FAILED"));
    assert!(rendered.contains("write temporary output result.json"));
    assert!(rendered.contains("disk unavailable"));
}

#[test]
fn atomic_write_reports_temporary_stage_creation_failures() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("missing").join("value.txt");

    let error = atomic_write(&destination, "value").unwrap_err();

    assert!(error.to_string().contains("WRITE_FAILED"));
    assert!(error.to_string().contains("create temporary output"));
    assert!(!destination.exists());
}

#[cfg(unix)]
#[test]
fn atomic_write_ignores_predictable_temps_and_preserves_replaced_stage_entries() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("value.txt");
    let predictable = temp
        .path()
        .join(format!(".value.txt.veac-tmp-{}-0", std::process::id()));
    std::fs::write(&predictable, b"sentinel").unwrap();
    atomic_write(&destination, "first").unwrap();
    assert_eq!(std::fs::read(&predictable).unwrap(), b"sentinel");

    let external = temp.path().join("external");
    std::fs::write(&external, b"external").unwrap();
    let mut replacement = None;
    let error = atomic_write_impl(&destination, "second", |stage| {
        let stage = stage.to_owned();
        std::fs::remove_file(&stage).unwrap();
        symlink(&external, &stage).unwrap();
        replacement = Some(stage);
    })
    .unwrap_err();
    let replacement = replacement.unwrap();

    assert!(error.to_string().contains("WRITE_FAILED"));
    assert!(std::fs::symlink_metadata(replacement)
        .unwrap()
        .file_type()
        .is_symlink());
    assert_eq!(std::fs::read(external).unwrap(), b"external");
    assert_eq!(std::fs::read(destination).unwrap(), b"first");
}

#[test]
fn atomic_write_distinguishes_an_error_after_the_commit_point() {
    let temp = tempfile::tempdir().unwrap();
    let destination = temp.path().join("value.txt");
    std::fs::write(&destination, b"old").unwrap();
    let mut replacement_directory = None;

    let error = atomic_write_impl(&destination, "new", |payload| {
        let visible = payload.parent().unwrap().to_owned();
        let moved = temp.path().join("moved-stage");
        std::fs::rename(&visible, moved).unwrap();
        std::fs::create_dir(&visible).unwrap();
        std::fs::write(visible.join("foreign"), b"foreign").unwrap();
        replacement_directory = Some(visible);
    })
    .unwrap_err();

    assert!(error.to_string().contains("WRITE_COMMIT_UNCERTAIN"));
    assert!(error.to_string().contains("do not retry blindly"));
    assert_eq!(std::fs::read(destination).unwrap(), b"new");
    assert_eq!(
        std::fs::read(replacement_directory.unwrap().join("foreign")).unwrap(),
        b"foreign"
    );
}

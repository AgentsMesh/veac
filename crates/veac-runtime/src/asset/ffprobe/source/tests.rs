use std::io;

use super::*;

#[test]
fn snapshot_helpers_preserve_io_context_and_reject_directories() {
    let temp = tempfile::tempdir().unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(1);
    let error = match ProbeSource::capture(temp.path(), deadline) {
        Err(error) => error,
        Ok(_) => panic!("directories must not become media snapshots"),
    };
    assert!(matches!(
        error,
        ProbeError::Io {
            operation: "snapshot",
            ref path,
            ..
        } if path == temp.path()
    ));

    let missing = temp.path().join("missing");
    assert_eq!(
        readonly(&missing).unwrap_err().kind(),
        io::ErrorKind::NotFound
    );
    let translated = io_error(
        "protect snapshot",
        &missing,
        io::Error::new(io::ErrorKind::PermissionDenied, "denied"),
    );
    assert!(matches!(
        translated,
        ProbeError::Io {
            operation: "protect snapshot",
            path,
            source,
        } if path == missing && source.kind() == io::ErrorKind::PermissionDenied
    ));
}

#[test]
fn expired_snapshot_deadline_remains_a_resource_error() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("media.bin");
    std::fs::write(&source, b"media").unwrap();
    let error = match ProbeSource::capture(&source, std::time::Instant::now()) {
        Err(error) => error,
        Ok(_) => panic!("expired capture must fail"),
    };
    assert!(matches!(
        error,
        ProbeError::ResourceLimit {
            operation: "source snapshot"
        }
    ));
}

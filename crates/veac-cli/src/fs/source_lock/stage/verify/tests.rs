use std::fs::{File, OpenOptions};
use std::path::Path;

use super::{content, ExpectedContent};

#[cfg(unix)]
#[test]
fn verification_maps_unseekable_descriptors() {
    let (stream, _peer) = std::os::unix::net::UnixStream::pair().unwrap();
    let descriptor: std::os::fd::OwnedFd = stream.into();
    let mut file = File::from(descriptor);

    let error = content(&mut file, &ExpectedContent::new(b""), Path::new("stage"))
        .expect_err("a socket cannot be rewound like a staged regular file");

    assert_eq!(error.diagnostics()[0].code, "WRITE_FAILED");
    assert!(error.to_string().contains("cannot verify staged source"));
}

#[test]
fn verification_maps_unreadable_descriptors() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    let mut file = OpenOptions::new().write(true).open(temp.path()).unwrap();

    let error = content(&mut file, &ExpectedContent::new(b""), Path::new("stage"))
        .expect_err("a write-only stage cannot be verified");

    assert_eq!(error.diagnostics()[0].code, "WRITE_FAILED");
    assert!(error.to_string().contains("cannot verify staged source"));
}

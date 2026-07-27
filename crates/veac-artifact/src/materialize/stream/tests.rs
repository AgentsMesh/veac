#![cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "tvos",
    target_os = "visionos",
    target_os = "watchos"
))]

use super::open_error;
use crate::ArtifactErrorKind;

#[test]
fn safe_open_errors_distinguish_symlink_loops_from_io_failures() {
    assert_eq!(
        open_error(rustix::io::Errno::LOOP).kind,
        ArtifactErrorKind::UnsafePath
    );
    assert_eq!(
        open_error(rustix::io::Errno::NOENT).kind,
        ArtifactErrorKind::Io
    );
}

#[test]
fn copy_hash_rejects_content_beyond_the_expected_size() {
    use std::io::{Seek, SeekFrom, Write};

    let mut input = tempfile::tempfile().unwrap();
    input.write_all(b"payload").unwrap();
    input.seek(SeekFrom::Start(0)).unwrap();
    let mut output = Vec::new();
    assert_eq!(
        super::copy_hash_while(&mut input, &mut output, 3, || true)
            .unwrap_err()
            .kind,
        ArtifactErrorKind::IdentityMismatch
    );
    assert!(output.is_empty());
}

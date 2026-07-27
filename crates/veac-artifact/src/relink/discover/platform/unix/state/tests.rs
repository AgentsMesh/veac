use std::error::Error;

use rustix::fs::FileType;

use super::*;

#[test]
fn relink_state_errors_preserve_path_and_io_classification() {
    let temp = tempfile::NamedTempFile::new().unwrap();
    let mismatch = inspect(temp.as_file(), FileType::Directory).unwrap_err();
    assert_eq!(mismatch.kind, ArtifactErrorKind::UnsafePath);

    let io = io_error(rustix::io::Errno::NOENT);
    assert_eq!(io.kind, ArtifactErrorKind::Io);
    assert!(io.source().is_some());
    assert_eq!(
        open_error(rustix::io::Errno::LOOP).kind,
        ArtifactErrorKind::UnsafePath
    );
    assert_eq!(
        unsafe_path::<()>("unsafe").unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
}

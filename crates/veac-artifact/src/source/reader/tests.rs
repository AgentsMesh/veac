use std::io::Write;

use super::*;

#[test]
fn bounded_reader_rechecks_growth_before_passing_bytes_to_the_consumer() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, b"a").unwrap();
    let mut consumed = false;

    let error = read_source_bounded(
        &source,
        None,
        1,
        || {
            std::fs::OpenOptions::new()
                .append(true)
                .open(&source)
                .unwrap()
                .write_all(b"bc")
                .unwrap();
        },
        |_| {
            consumed = true;
            Ok(())
        },
    )
    .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert!(!consumed);
}

#[test]
fn reader_size_accounting_rejects_overflow() {
    let error = checked_size(u64::MAX, 1).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
}

#[test]
fn declared_size_rejects_negative_filesystem_values() {
    let temp = tempfile::tempfile().unwrap();
    let mut state = rustix::fs::fstat(&temp).unwrap();
    state.st_size = -1;

    let error = file::declared_size(&state).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
}

#[test]
fn unidentified_confirmation_pass_cannot_read_past_the_hard_limit() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, b"a").unwrap();

    let error = read_source_bounded_guarded(
        &source,
        None,
        1,
        ReadHooks {
            after_open: || {},
            before_confirmation: || {
                std::fs::OpenOptions::new()
                    .append(true)
                    .open(&source)
                    .unwrap()
                    .write_all(b"bc")
                    .unwrap();
            },
            before_path_confirmation: || {},
        },
        || true,
        |_| Ok(()),
    )
    .unwrap_err();

    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
}

#[test]
fn path_confirmation_rejects_a_persistent_regular_file_replacement() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let original = temp.path().join("original");
    std::fs::write(&source, b"a").unwrap();

    let error = read_source_bounded_guarded(
        &source,
        None,
        1,
        ReadHooks {
            after_open: || {},
            before_confirmation: || {},
            before_path_confirmation: || {
                std::fs::rename(&source, &original).unwrap();
                std::fs::write(&source, b"b").unwrap();
            },
        },
        || true,
        |_| Ok(()),
    )
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
}

#[cfg(unix)]
#[test]
fn path_confirmation_rejects_a_symlink_replacement() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let original = temp.path().join("original");
    let target = temp.path().join("target");
    std::fs::write(&source, b"a").unwrap();
    std::fs::write(&target, b"a").unwrap();

    let error = read_source_bounded_guarded(
        &source,
        None,
        1,
        ReadHooks {
            after_open: || {},
            before_confirmation: || {},
            before_path_confirmation: || {
                std::fs::rename(&source, &original).unwrap();
                symlink(&target, &source).unwrap();
            },
        },
        || true,
        |_| Ok(()),
    )
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
}

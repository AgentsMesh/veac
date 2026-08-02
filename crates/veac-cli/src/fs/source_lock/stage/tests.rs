use std::path::Path;

use rustix::fd::OwnedFd;
use rustix::fs::Mode;

use super::super::path;
use super::Staged;

#[test]
fn stage_error_mappers_preserve_commit_state() {
    let io = super::io_error(
        Path::new("main.veac"),
        "write stage",
        std::io::Error::other("full"),
    );
    assert_eq!(io.diagnostics()[0].code, "WRITE_FAILED");
    let committed = super::committed(io);
    assert_eq!(committed.diagnostics()[0].code, "WRITE_COMMIT_UNCERTAIN");
}

#[test]
fn rename_error_after_replacement_reports_uncertainty() {
    let fixture = fixture();
    let staged = fixture.stage();

    let error = staged
        .publish_with(&fixture.parent, &fixture.source, rename_then_error)
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_COMMIT_UNCERTAIN");
    assert_eq!(std::fs::read_to_string(fixture.source).unwrap(), "after");
}

#[test]
fn rename_error_without_replacement_remains_uncommitted() {
    let fixture = fixture();
    let staged = fixture.stage();

    let error = staged
        .publish_with(&fixture.parent, &fixture.source, rename_error)
        .unwrap_err();

    assert_eq!(error.diagnostics()[0].code, "WRITE_FAILED");
    assert_eq!(std::fs::read_to_string(fixture.source).unwrap(), "before");
}

#[cfg(unix)]
#[test]
fn discard_maps_a_stage_unlink_permission_failure() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = fixture();
    let mut staged = fixture.stage();
    let root = fixture._temp.path();
    let original = std::fs::metadata(root).unwrap().permissions();
    std::fs::set_permissions(root, std::fs::Permissions::from_mode(0o500)).unwrap();

    let result = staged.discard();
    std::fs::set_permissions(root, original).unwrap();
    let error = result.expect_err("a read-only parent must reject stage removal");

    assert_eq!(error.diagnostics()[0].code, "WRITE_FAILED");
    assert!(error.to_string().contains("remove stage"));
}

struct Fixture {
    _temp: tempfile::TempDir,
    source: std::path::PathBuf,
    directory: OwnedFd,
    parent: path::Parent,
}

impl Fixture {
    fn stage(&self) -> Staged<'_> {
        Staged::create(
            &self.directory,
            Mode::RUSR | Mode::WUSR,
            b"after",
            &self.source,
        )
        .unwrap()
    }
}

fn fixture() -> Fixture {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("main.veac");
    std::fs::write(&source, "before").unwrap();
    let directory = rustix::fs::open(
        temp.path(),
        rustix::fs::OFlags::RDONLY | rustix::fs::OFlags::DIRECTORY,
        Mode::empty(),
    )
    .unwrap();
    let parent = path::resolve(&directory, "main.veac", &source).unwrap();
    Fixture {
        _temp: temp,
        source,
        directory,
        parent,
    }
}

fn rename_then_error(
    source_directory: &OwnedFd,
    source_name: &std::ffi::OsString,
    target_directory: &OwnedFd,
    target_name: &std::ffi::OsString,
) -> Result<(), rustix::io::Errno> {
    rustix::fs::renameat(source_directory, source_name, target_directory, target_name).unwrap();
    Err(rustix::io::Errno::IO)
}

fn rename_error(
    _source_directory: &OwnedFd,
    _source_name: &std::ffi::OsString,
    _target_directory: &OwnedFd,
    _target_name: &std::ffi::OsString,
) -> Result<(), rustix::io::Errno> {
    Err(rustix::io::Errno::IO)
}

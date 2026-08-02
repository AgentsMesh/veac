#![cfg(unix)]

use std::ffi::{OsStr, OsString};
use std::os::unix::ffi::OsStringExt;
use std::os::unix::fs::{symlink, PermissionsExt};
use std::time::{Duration, Instant};

use super::*;

#[test]
fn path_resolution_accepts_an_executable_symlink_target() {
    let temp = tempfile::tempdir().unwrap();
    let bin = temp.path().join("bin");
    std::fs::create_dir(&bin).unwrap();
    let target = script(temp.path().join("tool-real"), "#!/bin/sh\nexit 0\n");
    symlink(&target, bin.join("veac-path-tool")).unwrap();
    let search = std::env::join_paths([&bin]).unwrap();

    let resolved = resolve_in(Path::new("veac-path-tool"), &search).unwrap();
    assert_eq!(resolved, std::fs::canonicalize(target).unwrap());
}

#[test]
fn pinned_symlink_is_private_read_only_and_independent() {
    let temp = tempfile::tempdir().unwrap();
    let target = script(temp.path().join("target"), "#!/bin/sh\nexit 0\n");
    let link = temp.path().join("link");
    symlink(&target, &link).unwrap();

    let pinned = capture(&link).unwrap();
    assert_eq!(std::fs::read(&pinned.path).unwrap(), b"#!/bin/sh\nexit 0\n");
    assert_eq!(
        std::fs::metadata(&pinned.path)
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o400
    );
    let launch = launch(&pinned).unwrap();
    assert_eq!(
        std::fs::metadata(launch.path())
            .unwrap()
            .permissions()
            .mode()
            & 0o777,
        0o500
    );
    std::fs::write(target, b"changed").unwrap();
    assert_eq!(std::fs::read(&pinned.path).unwrap(), b"#!/bin/sh\nexit 0\n");
}

#[test]
fn dynamic_malformed_and_non_executable_tools_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    for (index, content, message) in [
        (0, "#!/usr/bin/env sh\nexit 0\n", "fixed absolute"),
        (1, "#!relative\nexit 0\n", "fixed absolute"),
        (2, "#!\nexit 0\n", "no interpreter"),
    ] {
        let path = script(temp.path().join(index.to_string()), content);
        assert!(capture(&path).unwrap_err().message.contains(message));
    }
    let non_utf8 = temp.path().join("non-utf8");
    std::fs::write(&non_utf8, [b'#', b'!', b'/', 0xff, b'\n']).unwrap();
    std::fs::set_permissions(&non_utf8, std::fs::Permissions::from_mode(0o755)).unwrap();
    assert!(capture(&non_utf8).is_err());

    let plain = temp.path().join("plain");
    std::fs::write(&plain, b"plain").unwrap();
    assert!(capture(&plain)
        .unwrap_err()
        .message
        .contains("not executable"));

    let interpreter = script(temp.path().join("mutable-sh"), "#!/bin/sh\nexit 0\n");
    let dynamic = script(
        temp.path().join("dynamic"),
        &format!("#!{}\nexit 0\n", interpreter.display()),
    );
    assert!(capture(&dynamic).unwrap_err().message.contains("writable"));
}

#[test]
fn missing_path_and_non_regular_targets_report_resolution_errors() {
    let missing = resolve_in(Path::new("missing"), OsStr::new("/no/such/path")).unwrap_err();
    assert!(missing.message.contains("on PATH"));

    let temp = tempfile::tempdir().unwrap();
    assert!(capture(temp.path()).is_err());
    let malformed = OsString::from_vec(vec![0xff]);
    assert!(resolve_in(Path::new("missing"), &malformed).is_err());

    let absolute = temp.path().join("absolute-missing");
    assert!(capture(&absolute)
        .unwrap_err()
        .message
        .contains("cannot resolve executable"));
}

#[test]
fn expired_deadlines_stop_tool_snapshot_and_launch_before_work() {
    let temp = tempfile::tempdir().unwrap();
    let target = script(temp.path().join("tool"), "#!/bin/sh\nexit 0\n");
    let error = PinnedExecutable::capture_until(&target, std::time::Instant::now()).unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);

    let pinned = capture(&target).unwrap();
    let error = pinned.launch_until(std::time::Instant::now()).unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);
}

#[test]
fn expired_cached_capture_does_not_poison_a_later_attempt() {
    let temp = tempfile::tempdir().unwrap();
    let target = script(temp.path().join("tool"), "#!/bin/sh\nexit 0\n");
    let cache = std::sync::OnceLock::new();
    let error =
        PinnedExecutable::cached_until(&cache, &target, std::time::Instant::now()).unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);

    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
    let pinned = PinnedExecutable::cached_until(&cache, &target, deadline).unwrap();
    assert_eq!(std::fs::read(&pinned.path).unwrap(), b"#!/bin/sh\nexit 0\n");
}

#[test]
fn interpreter_and_io_helpers_report_exact_failure_context() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing-interpreter");
    let missing_shebang = format!("#!{}\n", missing.display());
    assert!(interpreter::validate(missing_shebang.as_bytes())
        .unwrap_err()
        .message
        .contains("cannot resolve executable interpreter"));

    let non_executable = temp.path().join("non-executable-interpreter");
    std::fs::write(&non_executable, b"interpreter").unwrap();
    std::fs::set_permissions(&non_executable, std::fs::Permissions::from_mode(0o444)).unwrap();
    let non_executable_shebang = format!("#!{}\n", non_executable.display());
    assert!(interpreter::validate(non_executable_shebang.as_bytes())
        .unwrap_err()
        .message
        .contains("executable regular file"));

    let translated = interpreter::interpreter_error(std::io::Error::new(
        std::io::ErrorKind::PermissionDenied,
        "denied",
    ));
    assert!(translated
        .message
        .contains("cannot validate executable interpreter"));

    let io_error = io_result(
        Err(std::io::Error::new(std::io::ErrorKind::Other, "failed")),
        "exercise test operation",
    )
    .unwrap_err();
    assert!(io_error.message.contains("cannot exercise test operation"));
}

fn script(path: PathBuf, content: &str) -> PathBuf {
    std::fs::write(&path, content.as_bytes()).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

fn capture(path: &Path) -> Result<PinnedExecutable, RuntimeError> {
    PinnedExecutable::capture_until(path, deadline())
}

fn launch(pinned: &PinnedExecutable) -> Result<LaunchExecutable, RuntimeError> {
    pinned.launch_until(deadline())
}

fn deadline() -> Instant {
    Instant::now() + Duration::from_secs(10)
}

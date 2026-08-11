#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;

use super::*;

#[test]
fn fingerprint_binds_the_pinned_executable_and_version() {
    let temp = tempfile::tempdir().unwrap();
    let first_path = tool(temp.path(), "first");
    let second = tool(temp.path(), "second");

    let pinned = SystemFfprobe::new(&first_path);
    let first = pinned.fingerprint().unwrap();
    std::fs::write(
        &first_path,
        b"#!/bin/sh\nprintf 'ffprobe version changed\\n'\n",
    )
    .unwrap();
    let repeated = pinned.fingerprint().unwrap();
    let second = SystemFfprobe::new(second).fingerprint().unwrap();

    assert_eq!(first, repeated);
    assert_ne!(first.configuration, second.configuration);
    assert!(first.version.contains("first"));
    first.configuration.validate().unwrap();

    let error = SystemFfprobe::new(temp.path().join("missing"))
        .fingerprint()
        .unwrap_err();
    assert!(matches!(error, ProbeError::ProcessSpawn { .. }));
}

fn tool(root: &std::path::Path, version: &str) -> std::path::PathBuf {
    let path = root.join(version);
    std::fs::write(
        &path,
        format!("#!/bin/sh\nprintf 'ffprobe version {version}\\n'\n"),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o700)).unwrap();
    path
}

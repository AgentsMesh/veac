#![cfg(unix)]

use std::os::unix::fs::PermissionsExt;
use std::time::{Duration, Instant};

use super::*;

#[test]
fn expired_pin_preserves_resource_kind_without_poisoning_the_cache() {
    let temp = tempfile::tempdir().unwrap();
    let executable = temp.path().join("ffprobe");
    std::fs::write(&executable, b"#!/bin/sh\nexit 0\n").unwrap();
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).unwrap();
    let ffprobe = SystemFfprobe::new(executable);

    let error = ffprobe.pinned_until(Instant::now()).unwrap_err();
    assert_eq!(error.kind, crate::RuntimeErrorKind::ResourceLimit);
    assert!(error.message.contains("failed to run ffprobe"));

    let deadline = Instant::now() + Duration::from_secs(10);
    assert!(ffprobe.pinned_until(deadline).is_ok());
}

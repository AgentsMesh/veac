use super::*;
use crate::environment::SystemEnvironment;
use crate::unit_tests::support::FakeEnvironment;

#[test]
fn runtime_adapter_forwards_the_complete_capability_surface() {
    let environment = FakeEnvironment::success();
    let runtime = RuntimeEnvironment(&environment);

    assert_eq!(runtime.encoders().unwrap(), environment.encoders.clone());
    assert_eq!(runtime.muxers().unwrap(), environment.muxers.clone());
    assert_eq!(runtime.decoders().unwrap(), environment.decoders.clone());
    assert_eq!(runtime.demuxers().unwrap(), environment.demuxers.clone());
    assert_eq!(runtime.filters().unwrap(), environment.filters.clone());
    assert_eq!(
        runtime.hardware_backends().unwrap(),
        environment.hardware_backends.clone()
    );
    assert_eq!(
        runtime.hardware_devices().unwrap(),
        environment.hardware_devices.clone()
    );
    assert_eq!(
        runtime.fingerprint().unwrap().configuration,
        environment.fingerprint_configuration.clone()
    );
}

#[test]
fn runtime_adapter_preserves_setup_deadline_resource_kind() {
    let environment = FakeEnvironment::success();
    let runtime = RuntimeEnvironment(&environment);
    let fingerprint = runtime.fingerprint_until(Instant::now()).unwrap_err();
    let capability = runtime
        .capability_until(BackendCapabilityKind::Encoder, Instant::now())
        .unwrap_err();
    assert_eq!(
        fingerprint.kind,
        veac_runtime::RuntimeErrorKind::ResourceLimit
    );
    assert_eq!(
        capability.kind,
        veac_runtime::RuntimeErrorKind::ResourceLimit
    );
}

#[cfg(unix)]
#[test]
fn runtime_adapter_propagates_deadlines_into_system_ffmpeg_queries() {
    let temp = tempfile::tempdir().unwrap();
    let fingerprint_tool = sleeping_tool(temp.path(), "fingerprint.sh");
    let fingerprint = RuntimeEnvironment(&SystemEnvironment::new(fingerprint_tool))
        .fingerprint_until(deadline())
        .unwrap_err();
    assert_eq!(
        fingerprint.kind,
        veac_runtime::RuntimeErrorKind::ResourceLimit
    );

    let capability_tool = sleeping_tool(temp.path(), "capability.sh");
    let capability = RuntimeEnvironment(&SystemEnvironment::new(capability_tool))
        .capability_until(BackendCapabilityKind::Encoder, deadline())
        .unwrap_err();
    assert_eq!(
        capability.kind,
        veac_runtime::RuntimeErrorKind::ResourceLimit
    );
}

#[cfg(unix)]
fn sleeping_tool(root: &std::path::Path, name: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt;

    let path = root.join(name);
    std::fs::write(
        &path,
        "#!/bin/sh\nsleep 5\nprintf 'ffmpeg version late\\n'\n",
    )
    .unwrap();
    let mut permissions = std::fs::metadata(&path).unwrap().permissions();
    permissions.set_mode(0o700);
    std::fs::set_permissions(&path, permissions).unwrap();
    path
}

#[cfg(unix)]
fn deadline() -> Instant {
    Instant::now() + std::time::Duration::from_millis(300)
}

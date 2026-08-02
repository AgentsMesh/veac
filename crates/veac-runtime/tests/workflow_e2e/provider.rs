use veac_artifact::ArtifactStore;
use veac_provider::bind_provider_executable;
use veac_runtime::workflow::{ProviderResourceLimits, ProviderRunner, WorkflowErrorKind};

use super::provider_fixture::{fixture, provider_script};

#[cfg(unix)]
#[test]
fn provider_process_negotiates_uses_argv_and_commits_verified_payload() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let script = provider_script(temp.path(), &fixture);
    let store = ArtifactStore::new(temp.path().join("store"));
    let execution = ProviderRunner::new(&script)
        .with_arguments(["valid;literal"])
        .run(&store, &fixture.request)
        .unwrap();
    assert_eq!(execution.manifest, fixture.manifest);
    assert_eq!(execution.identity.provider, fixture.request.provider);
    assert_eq!(
        execution.response,
        bind_provider_executable(&fixture.response, execution.identity.executable.clone(),)
            .unwrap()
    );
    assert_eq!(execution.artifacts.len(), 1);
    assert_eq!(
        store
            .get(&execution.artifacts[0].key)
            .unwrap()
            .unwrap()
            .payload,
        b"speech"
    );
    let reused = ProviderRunner::new(&script)
        .with_arguments(["valid;literal"])
        .run(&store, &fixture.request)
        .unwrap();
    assert_eq!(reused.artifacts, execution.artifacts);
    assert_eq!(reused.identity, execution.identity);
    assert_eq!(reused.response, execution.response);
}

#[cfg(unix)]
#[test]
fn provider_process_enforces_output_timeout_and_staging_budgets_end_to_end() {
    use std::time::Duration;

    let cases = [
        ("stdout-oversize", resource_limits(4_096, 1_024, 1, 16)),
        ("stderr-oversize", resource_limits(16_384, 64, 1, 16)),
        ("timeout", resource_limits(16_384, 1_024, 0, 16)),
        ("valid", resource_limits(16_384, 1_024, 1, 5)),
    ];
    for (mode, limits) in cases {
        let temp = tempfile::tempdir().unwrap();
        let fixture = fixture();
        let script = provider_script(temp.path(), &fixture);
        let store = ArtifactStore::new(temp.path().join("store"));
        let error = ProviderRunner::new(script)
            .with_arguments([mode])
            .with_limits(limits)
            .run(&store, &fixture.request)
            .unwrap_err();
        assert!(
            matches!(
                error.kind,
                WorkflowErrorKind::ProtocolViolation | WorkflowErrorKind::ResourceLimit
            ),
            "case {mode}: {error}"
        );
        assert!(!store.root().exists(), "case {mode} committed output");
    }

    fn resource_limits(
        stdout: u64,
        stderr: u64,
        seconds: u64,
        payload: u64,
    ) -> ProviderResourceLimits {
        ProviderResourceLimits {
            max_stdout_bytes: stdout,
            max_stderr_bytes: stderr,
            max_wall_time: if seconds == 0 {
                Duration::from_millis(80)
            } else {
                Duration::from_secs(seconds)
            },
            max_payload_bytes: payload,
            max_total_payload_bytes: payload,
            ..ProviderResourceLimits::default()
        }
    }
}

#[cfg(unix)]
#[test]
fn provider_process_fails_closed_for_protocol_and_staging_attacks() {
    let cases = [
        ("manifest-polluted", WorkflowErrorKind::ProtocolViolation),
        ("polluted", WorkflowErrorKind::ProtocolViolation),
        ("missing", WorkflowErrorKind::UnsafeStaging),
        ("extra", WorkflowErrorKind::UnsafeStaging),
        ("symlink", WorkflowErrorKind::UnsafeStaging),
        ("directory", WorkflowErrorKind::UnsafeStaging),
        ("digest", WorkflowErrorKind::UnsafeStaging),
        ("nonzero", WorkflowErrorKind::ToolFailure),
    ];
    for (mode, expected) in cases {
        let temp = tempfile::tempdir().unwrap();
        let fixture = fixture();
        let script = provider_script(temp.path(), &fixture);
        let store = ArtifactStore::new(temp.path().join("store"));
        let error = ProviderRunner::new(script)
            .with_arguments([mode])
            .run(&store, &fixture.request)
            .unwrap_err();
        assert_eq!(error.kind, expected, "case {mode}: {error}");
        assert!(!store.root().exists());
    }
}

#[test]
fn provider_runner_rejects_invalid_requests_and_unavailable_programs() {
    let temp = tempfile::tempdir().unwrap();
    let mut value = fixture();
    value.request.schema = "invalid".into();
    assert_eq!(
        ProviderRunner::new(temp.path().join("missing"))
            .run(
                &ArtifactStore::new(temp.path().join("store")),
                &value.request,
            )
            .unwrap_err()
            .kind,
        WorkflowErrorKind::InvalidContract
    );

    value = fixture();
    assert_eq!(
        ProviderRunner::new(temp.path().join("missing"))
            .run(
                &ArtifactStore::new(temp.path().join("store")),
                &value.request,
            )
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ToolFailure
    );
}

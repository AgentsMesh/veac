use std::error::Error;
use std::time::Duration;

use veac_artifact::{ArtifactStore, ContentDigest};
use veac_provider::bind_provider_executable;

use super::super::*;
use super::support::{fixture, provider_script};

#[cfg(unix)]
#[test]
fn runner_negotiates_executes_and_commits_a_verified_payload() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let script = provider_script(temp.path(), &fixture);
    let store = ArtifactStore::new(temp.path().join("store"));
    let execution = ProviderRunner::new(&script)
        .with_arguments(["valid"])
        .run(&store, &fixture.request)
        .unwrap();
    assert_eq!(execution.manifest, fixture.manifest);
    assert_eq!(execution.identity.provider, fixture.request.provider);
    assert_eq!(
        execution.response,
        bind_provider_executable(&fixture.response, execution.identity.executable.clone(),)
            .unwrap()
    );
    assert_ne!(
        execution.response.output.artifacts()[0].record.key,
        fixture.response.output.artifacts()[0].record.key
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
        .with_arguments(["valid"])
        .run(&store, &fixture.request)
        .unwrap();
    assert_eq!(reused.artifacts, execution.artifacts);
    assert_eq!(reused.identity, execution.identity);
    assert_eq!(reused.response, execution.response);
}

#[cfg(unix)]
#[test]
fn runner_fails_closed_for_protocol_process_and_staging_attacks() {
    let cases = [
        ("manifest-polluted", WorkflowErrorKind::ProtocolViolation),
        ("manifest-nonzero", WorkflowErrorKind::ToolFailure),
        ("polluted", WorkflowErrorKind::ProtocolViolation),
        ("missing", WorkflowErrorKind::UnsafeStaging),
        ("extra", WorkflowErrorKind::UnsafeStaging),
        ("symlink", WorkflowErrorKind::UnsafeStaging),
        ("directory", WorkflowErrorKind::UnsafeStaging),
        ("size", WorkflowErrorKind::UnsafeStaging),
        ("digest", WorkflowErrorKind::UnsafeStaging),
        ("nonzero", WorkflowErrorKind::ToolFailure),
    ];
    for (mode, expected) in cases {
        let temp = tempfile::tempdir().unwrap();
        let fixture = fixture();
        let script = provider_script(temp.path(), &fixture);
        let error = ProviderRunner::new(script)
            .with_arguments([mode])
            .run(
                &ArtifactStore::new(temp.path().join("store")),
                &fixture.request,
            )
            .unwrap_err();
        assert_eq!(error.kind, expected, "case {mode}: {error}");
    }
}

#[cfg(unix)]
#[test]
fn runner_rejects_invalid_requests_and_mismatched_pins() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let store = ArtifactStore::new(temp.path().join("store"));
    let mut invalid = fixture.request.clone();
    invalid.schema = "invalid".into();
    let error = ProviderRunner::new("unused")
        .run(&store, &invalid)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::InvalidContract);
    assert!(error.source().is_some());

    let error = ProviderRunner::new(temp.path().join("missing-provider"))
        .run(&store, &fixture.request)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);

    let script = provider_script(temp.path(), &fixture);
    let mut mismatched = fixture.request.clone();
    mismatched.provider.provider = "other-provider".into();
    let error = ProviderRunner::new(&script)
        .with_arguments(["valid"])
        .run(&store, &mismatched)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ProtocolViolation);

    let mut changed = fixture.request.clone();
    let veac_provider::ProviderRequest::TextToSpeech(request) = &mut changed.request else {
        unreachable!()
    };
    request.text = "different request".into();
    let error = ProviderRunner::new(&script)
        .with_arguments(["response-mismatch"])
        .run(&store, &changed)
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ProtocolViolation);
    assert!(store
        .get(&ContentDigest::sha256(b"not-an-artifact-key"))
        .unwrap()
        .is_none());
}

#[cfg(unix)]
#[test]
fn manifest_and_execution_share_one_wall_clock_deadline() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let script = provider_script(temp.path(), &fixture);
    let limits = ProviderResourceLimits {
        max_wall_time: Duration::from_millis(300),
        ..ProviderResourceLimits::default()
    };
    let started = std::time::Instant::now();
    let error = ProviderRunner::new(script)
        .with_arguments(["shared-timeout"])
        .with_limits(limits)
        .run(
            &ArtifactStore::new(temp.path().join("store")),
            &fixture.request,
        )
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(started.elapsed() < Duration::from_millis(500));
}

#[cfg(unix)]
#[test]
fn tool_snapshot_obeys_the_provider_deadline() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let script = provider_script(temp.path(), &fixture);
    let limits = ProviderResourceLimits {
        max_wall_time: Duration::from_nanos(1),
        ..ProviderResourceLimits::default()
    };
    let error = ProviderRunner::new(script)
        .with_limits(limits)
        .run(
            &ArtifactStore::new(temp.path().join("store")),
            &fixture.request,
        )
        .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ResourceLimit);
    assert!(error.to_string().contains("pin provider executable"));
}

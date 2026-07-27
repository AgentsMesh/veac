use veac_provider::canonical_request_bytes;

use std::time::{Duration, Instant};

use super::super::{process, ProviderResourceLimits, WorkflowErrorKind};
use super::support::{fixture, provider_script};

#[cfg(unix)]
#[test]
fn provider_process_reports_start_failure_and_stdout_limit() {
    let temp = tempfile::tempdir().unwrap();
    let missing = temp.path().join("missing-provider");
    let limits = ProviderResourceLimits::default();
    let error = process::manifest(&missing, &[], limits, deadline(limits)).unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
    let error = process::execute(
        &missing,
        &[],
        temp.path(),
        b"request",
        limits,
        deadline(limits),
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);

    let fixture = fixture();
    let script = provider_script(temp.path(), &fixture);
    let error = process::manifest(
        &script,
        &["manifest-oversize".into()],
        limits,
        deadline(limits),
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ProtocolViolation);

    let request = canonical_request_bytes(&fixture.request).unwrap();
    let error = process::execute(
        &script,
        &["nonzero".into()],
        temp.path(),
        &request,
        limits,
        deadline(limits),
    )
    .unwrap_err();
    assert_eq!(error.kind, WorkflowErrorKind::ToolFailure);
}

#[cfg(unix)]
#[test]
fn provider_process_enforces_configured_output_request_and_wall_limits() {
    let temp = tempfile::tempdir().unwrap();
    let fixture = fixture();
    let script = provider_script(temp.path(), &fixture);
    let limits = ProviderResourceLimits {
        max_stdout_bytes: 64,
        ..ProviderResourceLimits::default()
    };
    assert_eq!(
        process::manifest(&script, &[], limits, deadline(limits))
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ProtocolViolation
    );

    let limits = ProviderResourceLimits {
        max_stderr_bytes: 64,
        ..ProviderResourceLimits::default()
    };
    assert_eq!(
        process::manifest(
            &script,
            &["manifest-stderr".into()],
            limits,
            deadline(limits),
        )
        .unwrap_err()
        .kind,
        WorkflowErrorKind::ResourceLimit
    );

    let limits = ProviderResourceLimits {
        max_wall_time: Duration::from_millis(50),
        ..ProviderResourceLimits::default()
    };
    assert_eq!(
        process::manifest(
            &script,
            &["manifest-timeout".into()],
            limits,
            deadline(limits),
        )
        .unwrap_err()
        .kind,
        WorkflowErrorKind::ResourceLimit
    );

    let limits = ProviderResourceLimits {
        max_request_bytes: 1,
        ..ProviderResourceLimits::default()
    };
    assert_eq!(
        process::execute(
            &script,
            &[],
            temp.path(),
            b"too large",
            limits,
            deadline(limits),
        )
        .unwrap_err()
        .kind,
        WorkflowErrorKind::ResourceLimit
    );

    for invalid in [
        Instant::now(),
        Instant::now() + ProviderResourceLimits::default().max_wall_time + Duration::from_secs(1),
    ] {
        assert_eq!(
            process::manifest(&script, &[], ProviderResourceLimits::default(), invalid,)
                .unwrap_err()
                .kind,
            WorkflowErrorKind::InvalidContract
        );
    }
}

fn deadline(limits: ProviderResourceLimits) -> Instant {
    Instant::now() + limits.max_wall_time
}

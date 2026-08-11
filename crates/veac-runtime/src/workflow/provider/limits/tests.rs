use super::*;

#[test]
fn zero_or_excessive_provider_limits_are_rejected() {
    let hard = ProviderResourceLimits::default();
    let invalid = [
        ProviderResourceLimits {
            max_request_bytes: 0,
            ..hard
        },
        ProviderResourceLimits {
            max_stdout_bytes: hard.max_stdout_bytes + 1,
            ..hard
        },
        ProviderResourceLimits {
            max_stderr_bytes: 0,
            ..hard
        },
        ProviderResourceLimits {
            max_wall_time: Duration::ZERO,
            ..hard
        },
        ProviderResourceLimits {
            max_artifacts: 0,
            ..hard
        },
        ProviderResourceLimits {
            max_payload_bytes: 0,
            ..hard
        },
        ProviderResourceLimits {
            max_total_payload_bytes: 0,
            ..hard
        },
    ];
    for limits in invalid {
        assert_eq!(
            limits.validate().unwrap_err().kind,
            WorkflowErrorKind::InvalidContract
        );
    }
    assert!(hard.validate().is_ok());
}

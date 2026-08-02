use super::*;

#[path = "tests/contract_cases.rs"]
mod contract_cases;
#[path = "tests/payload_cases.rs"]
mod payload_cases;
#[path = "tests/profile_cases.rs"]
mod profile_cases;
#[path = "tests/snapshot_cases.rs"]
mod snapshot_cases;
#[path = "tests/support.rs"]
mod support;
#[cfg(unix)]
#[path = "tests/validator_cases.rs"]
mod validator_cases;

#[test]
fn configured_limits_must_stay_within_hard_policy() {
    for (wall, bytes) in [
        (0, 1),
        (1, 0),
        (veac_artifact::MAX_MEDIA_DERIVATION_WALL_SECONDS + 1, 1),
        (1, veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES + 1),
    ] {
        let error = FullRenderSegmentValidator::with_limits(SystemFfprobe::default(), wall, bytes)
            .unwrap_err();
        assert_eq!(error.kind, WorkflowErrorKind::InvalidContract);
    }
    FullRenderSegmentValidator::with_limits(SystemFfprobe::default(), 1, 1).unwrap();
}

#[test]
fn stream_intent_is_exact_about_audio_presence() {
    assert_eq!(stream_intent(false).audio, StreamChoice::Disabled);
    assert_eq!(stream_intent(true).audio, StreamChoice::Auto);
}

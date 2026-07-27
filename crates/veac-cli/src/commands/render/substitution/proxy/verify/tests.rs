use super::*;
use std::time::Duration;
use veac_artifact::{
    ArtifactDescriptor, ArtifactKind, ArtifactStore, ContentDigest, ProducerFingerprint,
};

#[test]
fn proxy_probe_errors_gain_postflight_context_without_losing_resource_kind() {
    let failure = probe_error(CliError::new("PROBE_FAILED", "invalid media"));
    assert!(!failure.is_resource_limit());
    assert!(failure.to_string().contains("PROXY_POSTFLIGHT_FAILED"));
    assert!(failure.to_string().contains("PROBE_FAILED"));

    let limit = probe_error(CliError::resource_limit("PROBE_FAILED", "deadline"));
    assert!(limit.is_resource_limit());
    assert!(limit.to_string().contains("PROXY_POSTFLIGHT_FAILED"));
    assert!(limit.to_string().contains("PROBE_FAILED"));
}

#[test]
fn malformed_descriptors_and_payload_failures_are_typed() {
    let temp = tempfile::tempdir().unwrap();
    let store = ArtifactStore::new(temp.path().join("store"));
    let record = store.put(&descriptor(), b"payload").unwrap();
    let artifact = store.open(&record.key).unwrap().unwrap();
    let environment = crate::unit_tests::support::FakeEnvironment::success();
    let future = Instant::now() + Duration::from_secs(2);

    let malformed = validate(&artifact, Role::Video, &environment, future).unwrap_err();
    assert!(malformed.to_string().contains("PROXY_POSTFLIGHT_FAILED"));

    let expected = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: record.content.value,
    };
    let expired = verify_payload(&artifact, &expected, Instant::now()).unwrap_err();
    assert!(expired.is_resource_limit());
    std::fs::write(artifact.payload_path(), b"corrupt").unwrap();
    let corrupt = verify_payload(&artifact, &expected, future).unwrap_err();
    assert!(!corrupt.is_resource_limit());
}

#[test]
fn selection_and_role_defenses_preserve_error_classification() {
    let invalid = ContentDigest {
        algorithm: veac_artifact::DigestAlgorithm::Sha256,
        value: "invalid".into(),
    }
    .validate()
    .unwrap_err();
    assert!(!selection_error(invalid).is_resource_limit());

    let temp = tempfile::tempdir().unwrap();
    let payload = temp.path().join("payload");
    std::fs::write(&payload, b"payload").unwrap();
    let limit = verify_source_bounded_while(&payload, None, 64, || false).unwrap_err();
    assert!(selection_error(limit).is_resource_limit());
    assert_eq!(intent(Role::Audio).audio, StreamChoice::Auto);
    let video = MediaArtifactSpec::ProxyVideo(veac_artifact::ProxyVideoSpec {
        source_stream: veac_ir::StreamSelection {
            global_index: 0,
            type_index: 0,
        },
        source_clock: veac_artifact::SourceClockSpec::Identity {
            duration: veac_ir::RationalTime::new(1, 1).unwrap(),
        },
        width: 32,
        height: 24,
        frame_rate: veac_ir::Rational::new(10, 1).unwrap(),
        crf: 28,
    });
    assert!(!role_matches(Role::Audio, &video));
    assert!(check_deadline(Instant::now())
        .unwrap_err()
        .is_resource_limit());
    assert!(!postflight_error("postflight").is_resource_limit());
    assert!(resource_error("resource").is_resource_limit());
}

fn descriptor() -> ArtifactDescriptor {
    ArtifactDescriptor::new(
        ArtifactKind::Analysis,
        ProducerFingerprint {
            name: "proxy-test".into(),
            version: "1".into(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
        vec![],
        serde_json::json!({}),
    )
}

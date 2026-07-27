use serde_json::json;
use veac_artifact::{ArtifactDependency, ArtifactKind, ContentDigest};

use crate::test_support::*;
use crate::*;

#[test]
fn every_request_is_canonical_versioned_and_round_trips() {
    let schema = provider_request_json_schema().unwrap();
    assert_eq!(schema["title"], "ProviderRequestEnvelope");
    assert!(provider_response_json_schema().unwrap().is_object());
    assert!(provider_manifest_json_schema().unwrap().is_object());
    assert!(provider_edit_proposal_json_schema().unwrap().is_object());
    for payload in requests() {
        let envelope = ProviderRequestEnvelope::new(negotiated(payload.capability()), payload)
            .expect("valid request");
        let bytes = canonical_request_bytes(&envelope).unwrap();
        let decoded: ProviderRequestEnvelope = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, envelope);
        assert_eq!(
            request_hash(&decoded).unwrap(),
            request_hash(&envelope).unwrap()
        );
        assert!(String::from_utf8(bytes).unwrap().starts_with('{'));
    }
}

#[test]
fn every_output_is_bound_to_request_and_has_stable_identity() {
    for payload in requests() {
        let request =
            ProviderRequestEnvelope::new(negotiated(payload.capability()), payload).unwrap();
        let response = ProviderResponseEnvelope::new(&request, output_for(&request)).unwrap();
        response.validate_for(&request).unwrap();
        let bytes = canonical_response_bytes(&response).unwrap();
        let decoded: ProviderResponseEnvelope = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(decoded, response);
        assert_eq!(
            response_hash(&decoded).unwrap(),
            response_hash(&response).unwrap()
        );
    }
}

#[test]
fn request_hash_changes_with_semantics_and_provider_pin() {
    let payload = requests().remove(0);
    let original = ProviderRequestEnvelope::new(negotiated(payload.capability()), payload).unwrap();
    let mut changed = original.clone();
    changed.provider.configuration = ContentDigest::sha256(b"different configuration");
    assert_ne!(
        request_hash(&original).unwrap(),
        request_hash(&changed).unwrap()
    );
}

#[test]
fn envelopes_reject_wrong_schema_version_capability_and_request() {
    let payload = requests().remove(0);
    let mut request =
        ProviderRequestEnvelope::new(negotiated(payload.capability()), payload).unwrap();
    request.schema = "future".into();
    assert!(request.validate().is_err());
    request.schema = PROVIDER_SCHEMA_ID.into();
    request.contract_version = 9;
    assert!(request.validate().is_err());
    request.contract_version = 1;
    request.capability = Capability::Translation;
    assert!(request.validate().is_err());

    let payload = requests().remove(0);
    let wrong = negotiated(Capability::Translation);
    assert!(ProviderRequestEnvelope::new(wrong, payload).is_err());
}

#[test]
fn response_rejects_mismatch_and_unbound_artifacts() {
    let payload = requests().remove(3);
    let request = ProviderRequestEnvelope::new(negotiated(payload.capability()), payload).unwrap();
    let mut response = ProviderResponseEnvelope::new(&request, output_for(&request)).unwrap();
    response.request_hash = ContentDigest::sha256(b"wrong request");
    assert!(response.validate_for(&request).is_err());

    response = ProviderResponseEnvelope::new(&request, output_for(&request)).unwrap();
    response.capability = Capability::Dubbing;
    assert!(response.validate().is_err());

    response = ProviderResponseEnvelope::new(&request, output_for(&request)).unwrap();
    if let ProviderOutput::TextToSpeech(value) = &mut response.output {
        let mut other = request.provider.clone();
        other.provider = "other-provider".into();
        value.audio = artifact(
            ArtifactKind::Speech,
            "speech",
            &other,
            request_hash(&request).unwrap(),
        );
    }
    assert!(response.validate().is_err());

    response = ProviderResponseEnvelope::new(&request, output_for(&request)).unwrap();
    if let ProviderOutput::TextToSpeech(value) = &mut response.output {
        value.audio = artifact(
            ArtifactKind::Speech,
            "speech",
            &request.provider,
            ContentDigest::sha256(b"other request"),
        );
    }
    assert!(response.validate().is_err());
}

#[test]
fn artifact_descriptor_sorts_dependencies_and_record_checks_key() {
    let provider = fingerprint();
    let request = ContentDigest::sha256(b"request");
    let descriptor = provider_artifact_descriptor(
        ArtifactKind::Analysis,
        &provider,
        request.clone(),
        vec![
            ArtifactDependency {
                role: "z".into(),
                identity: ContentDigest::sha256(b"z"),
            },
            ArtifactDependency {
                role: "a".into(),
                identity: ContentDigest::sha256(b"a"),
            },
        ],
        json!({"threshold": 0.5}),
    )
    .unwrap();
    assert_eq!(descriptor.dependencies[0].role, "a");
    let mut value = artifact(ArtifactKind::Analysis, "analysis", &provider, request);
    assert_eq!(value.kind(), ArtifactKind::Analysis);
    value.record.key = ContentDigest::sha256(b"wrong");
    assert!(value.validate().is_err());
    value.role.clear();
    assert!(value.validate().is_err());

    value = artifact(
        ArtifactKind::Analysis,
        "analysis",
        &provider,
        ContentDigest::sha256(b"request-2"),
    );
    value.record.size_bytes = veac_artifact::MAX_ARTIFACT_PAYLOAD_BYTES + 1;
    assert!(value.validate().is_err());
}

#[test]
fn tagged_payloads_reject_unknown_fields() {
    let payload = requests().remove(0);
    let request = ProviderRequestEnvelope::new(negotiated(payload.capability()), payload).unwrap();
    let mut value = serde_json::to_value(request).unwrap();
    value["request"]["unexpected"] = json!(true);
    assert!(serde_json::from_value::<ProviderRequestEnvelope>(value).is_err());
}

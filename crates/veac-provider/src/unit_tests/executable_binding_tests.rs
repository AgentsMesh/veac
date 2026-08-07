use veac_artifact::{artifact_key, ArtifactDependency, ContentDigest};

use crate::test_support::{negotiated, output_for, requests};
use crate::*;

#[test]
fn executable_binding_covers_every_artifact_output_and_is_stable() {
    let executable = ContentDigest::sha256(b"provider executable");
    let mut artifact_count = 0;
    for request in requests() {
        let envelope =
            ProviderRequestEnvelope::new(negotiated(request.capability()), request).unwrap();
        let raw = ProviderResponseEnvelope::new(&envelope, output_for(&envelope)).unwrap();
        let bound = bind_provider_executable(&raw, executable.clone()).unwrap();
        assert_eq!(
            bind_provider_executable(&raw, executable.clone()).unwrap(),
            bound
        );
        assert_eq!(bound.provider, raw.provider);
        assert_eq!(bound.request_hash, raw.request_hash);
        bound.validate_for(&envelope).unwrap();
        let raw_artifacts = raw.output.artifacts();
        let bound_artifacts = bound.output.artifacts();
        assert_eq!(raw_artifacts.len(), bound_artifacts.len());
        for (raw_artifact, bound_artifact) in raw_artifacts.into_iter().zip(bound_artifacts) {
            artifact_count += 1;
            assert_ne!(bound_artifact.record.key, raw_artifact.record.key);
            assert_eq!(bound_artifact.record.content, raw_artifact.record.content);
            assert_eq!(
                bound_artifact.record.size_bytes,
                raw_artifact.record.size_bytes
            );
            let dependencies = bound_artifact
                .descriptor
                .dependencies
                .iter()
                .filter(|value| value.role == PROVIDER_EXECUTABLE_DEPENDENCY_ROLE)
                .collect::<Vec<_>>();
            assert_eq!(dependencies.len(), 1);
            assert_eq!(dependencies[0].identity, executable);
        }
        if !raw.output.artifacts().is_empty() {
            assert_ne!(response_hash(&raw).unwrap(), response_hash(&bound).unwrap());
        }
    }
    assert_eq!(artifact_count, 10);
}

#[test]
fn executable_binding_rejects_a_provider_declared_reserved_dependency() {
    let request = requests()
        .into_iter()
        .find(|value| value.capability() == Capability::TextToSpeech)
        .unwrap();
    let envelope = ProviderRequestEnvelope::new(negotiated(request.capability()), request).unwrap();
    let mut raw = ProviderResponseEnvelope::new(&envelope, output_for(&envelope)).unwrap();
    let artifact = raw.output.artifacts_mut().into_iter().next().unwrap();
    artifact
        .descriptor
        .dependencies
        .push(ArtifactDependency::new(
            PROVIDER_EXECUTABLE_DEPENDENCY_ROLE,
            ContentDigest::sha256(b"provider supplied identity"),
        ));
    artifact.descriptor.dependencies.sort_by(|left, right| {
        (&left.role, &left.identity.value).cmp(&(&right.role, &right.identity.value))
    });
    artifact.record.key = artifact_key(&artifact.descriptor).unwrap();
    raw.validate_for(&envelope).unwrap();
    let error =
        bind_provider_executable(&raw, ContentDigest::sha256(b"host identity")).unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::InvalidContract);
    assert!(error.to_string().contains("reserved provider_executable"));
}

#[test]
fn executable_binding_rejects_an_invalid_host_digest() {
    let request = requests().remove(0);
    let envelope = ProviderRequestEnvelope::new(negotiated(request.capability()), request).unwrap();
    let raw = ProviderResponseEnvelope::new(&envelope, output_for(&envelope)).unwrap();
    let error = bind_provider_executable(
        &raw,
        ContentDigest {
            algorithm: veac_artifact::DigestAlgorithm::Sha256,
            value: "invalid".to_owned(),
        },
    )
    .unwrap_err();
    assert_eq!(error.kind, ProviderErrorKind::Artifact);
}

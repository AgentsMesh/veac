use crate::{
    ArtifactEvidenceOutcome, ArtifactKind, ArtifactParameters, ContentDigest,
    EvidenceBundleParameters,
};

#[test]
fn evidence_bundle_parameters_are_closed_content_identity() {
    let suite = ContentDigest::sha256(b"suite").value;
    let report = ContentDigest::sha256(b"report").value;
    for outcome in [
        ArtifactEvidenceOutcome::Pass,
        ArtifactEvidenceOutcome::Fail,
        ArtifactEvidenceOutcome::Error,
    ] {
        let value = ArtifactParameters::EvidenceBundle(EvidenceBundleParameters::new(
            &suite, &report, outcome,
        ));
        assert_eq!(value.kind(), ArtifactKind::EvidenceBundle);
        assert!(value.validate().is_ok());
        let encoded = serde_json::to_value(&value).unwrap();
        assert_eq!(encoded["type"], "evidence_bundle");
        assert!(serde_json::from_value::<ArtifactParameters>(encoded).is_ok());
    }
}

#[test]
fn evidence_bundle_rejects_non_sha256_identities() {
    let valid = ContentDigest::sha256(b"valid").value;
    let invalid = ArtifactParameters::EvidenceBundle(EvidenceBundleParameters::new(
        "not-a-digest",
        valid,
        ArtifactEvidenceOutcome::Pass,
    ));
    assert!(invalid.validate().is_err());
}

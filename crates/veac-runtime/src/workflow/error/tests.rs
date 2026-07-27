use std::error::Error;

use veac_artifact::{ArtifactStore, ContentDigest, DigestAlgorithm};
use veac_provider::{
    negotiate, Capability, CapabilityRequirement, ProviderFingerprint, ProviderManifest,
};

use super::super::*;

#[test]
fn workflow_errors_preserve_kinds_messages_and_sources() {
    let plain = WorkflowError::new(WorkflowErrorKind::UnsupportedOperation, "unsupported");
    assert_eq!(plain.to_string(), "unsupported");
    assert!(plain.source().is_none());
    let sourced = WorkflowError::with_source(
        WorkflowErrorKind::Io,
        "wrapped",
        std::io::Error::other("cause"),
    );
    assert_eq!(sourced.to_string(), "wrapped");
    assert_eq!(sourced.source().unwrap().to_string(), "cause");

    let io = WorkflowError::from(std::io::Error::other("io"));
    assert_eq!(io.kind, WorkflowErrorKind::Io);
    assert!(io.source().is_some());

    let temp = tempfile::tempdir().unwrap();
    let invalid_digest = ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: "invalid".into(),
    };
    let artifact = ArtifactStore::new(temp.path())
        .get(&invalid_digest)
        .unwrap_err();
    let artifact = WorkflowError::from(artifact);
    assert_eq!(artifact.kind, WorkflowErrorKind::Artifact);
    assert!(artifact.source().is_some());

    let manifest = ProviderManifest::new(
        ProviderFingerprint {
            provider: "provider".into(),
            implementation_version: "1".into(),
            model: "model".into(),
            model_version: "1".into(),
            configuration: ContentDigest::sha256(b"configuration"),
        },
        Vec::new(),
    );
    let provider =
        negotiate(&manifest, &CapabilityRequirement::current(Capability::Asr)).unwrap_err();
    let provider = WorkflowError::from(provider);
    assert_eq!(provider.kind, WorkflowErrorKind::Provider);
    assert!(provider.source().is_some());
}

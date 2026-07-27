use veac_artifact::ContentDigest;
use veac_ir::{HashAlgorithm, MediaIdentity};

use super::*;

#[test]
fn provider_identity_accepts_sha256_and_rejects_other_algorithms() {
    let fingerprint = ProviderFingerprint {
        provider: "provider".into(),
        implementation_version: "1".into(),
        model: "model".into(),
        model_version: "1".into(),
        configuration: ContentDigest::sha256(b"configuration"),
    };
    let sha = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: "ab".repeat(32),
    };
    assert_eq!(
        ProviderExecutionIdentity::from_pinned(&fingerprint, &sha)
            .unwrap()
            .executable
            .value,
        sha.digest
    );

    let unsupported = MediaIdentity {
        algorithm: HashAlgorithm::Blake3,
        digest: "cd".repeat(32),
    };
    assert_eq!(
        ProviderExecutionIdentity::from_pinned(&fingerprint, &unsupported)
            .unwrap_err()
            .kind,
        WorkflowErrorKind::ToolFailure
    );
}

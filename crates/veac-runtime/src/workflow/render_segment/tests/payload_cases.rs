use std::time::{Duration, Instant};

use veac_ir::{HashAlgorithm, MediaIdentity};

use super::support::*;
use super::*;

#[test]
fn payload_verification_maps_success_size_identity_limit_and_io() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("segment.bin");
    let bytes = b"render segment";
    std::fs::write(&path, bytes).unwrap();
    let contract = contract(false);
    let mut expected = record(&contract);
    expected.size_bytes = bytes.len() as u64;
    let deadline = Instant::now() + Duration::from_secs(1);

    let verified = payload::verify(
        &path,
        &identity(&expected),
        &expected,
        bytes.len() as u64,
        deadline,
    )
    .unwrap();
    assert_eq!(verified.verified.size_bytes, bytes.len() as u64);
    assert_eq!(verified.prefix, bytes);

    let mut wrong_size = expected.clone();
    wrong_size.size_bytes += 1;
    assert_kind(
        payload::verify(&path, &identity(&expected), &wrong_size, 100, deadline),
        WorkflowErrorKind::SourceIdentityMismatch,
    );
    let wrong_identity = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: "ab".repeat(32),
    };
    assert_kind(
        payload::verify(&path, &wrong_identity, &expected, 100, deadline),
        WorkflowErrorKind::SourceIdentityMismatch,
    );
    assert_kind(
        payload::verify(&path, &identity(&expected), &expected, 1, deadline),
        WorkflowErrorKind::ResourceLimit,
    );
    assert_kind(
        payload::verify(
            &temp.path().join("missing.bin"),
            &identity(&expected),
            &expected,
            100,
            deadline,
        ),
        WorkflowErrorKind::Artifact,
    );
    assert_kind(
        payload::verify(
            &path,
            &identity(&expected),
            &expected,
            100,
            Instant::now() - Duration::from_millis(1),
        ),
        WorkflowErrorKind::ResourceLimit,
    );
}

fn assert_kind<T: std::fmt::Debug>(result: WorkflowResult<T>, expected: WorkflowErrorKind) {
    assert_eq!(result.unwrap_err().kind, expected);
}

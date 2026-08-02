use veac_ir::{HashAlgorithm, MediaIdentity};

use crate::{read_verified_source_bounded, ArtifactErrorKind};

#[test]
fn bounded_verified_reads_accept_the_exact_limit_and_reject_larger_sources() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, b"12345").unwrap();
    let expected = identity(b"12345");

    let bytes = read_verified_source_bounded(&source, Some(&expected), 5).unwrap();
    assert_eq!(bytes, b"12345");
    let error = read_verified_source_bounded(&source, Some(&expected), 4).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
}

#[test]
fn bounded_verified_reads_still_validate_the_declared_identity() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, b"12345").unwrap();

    let error = read_verified_source_bounded(&source, Some(&identity(b"wrong")), 5).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch);
}

#[test]
fn bounded_verified_reads_reject_malformed_identities_and_missing_paths() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, b"12345").unwrap();
    let malformed = MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: "ABC".to_owned(),
    };

    let error = read_verified_source_bounded(&source, Some(&malformed), 5).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
    let error = read_verified_source_bounded(&temp.path().join("missing"), None, 5).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::Io);
}

#[test]
fn public_source_limits_cannot_disable_the_hard_caps() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    assert_eq!(crate::MAX_RENDER_TASK_OUTPUT_BYTES, 64 * 1024 * 1024 * 1024);
    std::fs::write(&source, b"x").unwrap();
    for limit in [0, crate::MAX_IN_MEMORY_ARTIFACT_BYTES + 1, u64::MAX] {
        assert_eq!(
            read_verified_source_bounded(&source, None, limit)
                .unwrap_err()
                .kind,
            ArtifactErrorKind::InvalidContract
        );
    }
    for limit in [0, crate::MAX_VERIFIED_SOURCE_BYTES + 1, u64::MAX] {
        assert_eq!(
            crate::verify_source_bounded(&source, None, limit)
                .unwrap_err()
                .kind,
            ArtifactErrorKind::InvalidContract
        );
        assert_eq!(
            crate::copy_verified_source_bounded(
                &source,
                &temp.path().join(format!("copy-{limit}")),
                None,
                limit,
            )
            .unwrap_err()
            .kind,
            ArtifactErrorKind::InvalidContract
        );
    }
}

#[test]
fn caller_guards_abort_verification_and_copy_without_publishing() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let destination = temp.path().join("copy");
    std::fs::write(&source, vec![0x5a; 128 * 1024]).unwrap();
    let mut checks = 0;
    let error = crate::verify_source_bounded_while(&source, None, 128 * 1024, || {
        checks += 1;
        checks < 4
    })
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);

    let error =
        crate::copy_verified_source_bounded_while(&source, &destination, None, 128 * 1024, || {
            false
        })
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert!(!destination.exists());
}

#[test]
fn verified_prefix_hashes_the_whole_file_but_retains_only_the_bound() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, b"header-and-payload").unwrap();

    let result = crate::verify_source_prefix_bounded_while(&source, None, 18, 6, || true).unwrap();
    assert_eq!(result.prefix, b"header");
    assert_eq!(result.verified.size_bytes, 18);
    assert_eq!(result.verified.identity, identity(b"header-and-payload"));
}

#[test]
fn verified_prefix_limit_and_guard_fail_closed() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, b"source").unwrap();

    let error = crate::verify_source_prefix_bounded_while(
        &source,
        None,
        6,
        crate::MAX_VERIFIED_SOURCE_PREFIX_BYTES + 1,
        || true,
    )
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
    let error =
        crate::verify_source_prefix_bounded_while(&source, None, 6, 4, || false).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
}

#[cfg(unix)]
#[test]
fn verified_prefix_rejects_or_survives_a_to_b_to_a_path_swaps() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    let saved_a = temp.path().join("saved-a");
    let saved_b = temp.path().join("saved-b");
    let transient_b = temp.path().join("transient-b");
    std::fs::write(&source, b"version-a").unwrap();
    std::fs::write(&saved_b, b"version-b").unwrap();
    let mut checks = 0;

    let result = crate::verify_source_prefix_bounded_while(&source, None, 9, 9, || {
        checks += 1;
        if checks == 2 {
            std::fs::rename(&source, &saved_a).unwrap();
            std::fs::rename(&saved_b, &source).unwrap();
            std::fs::rename(&source, &transient_b).unwrap();
            std::fs::rename(&saved_a, &source).unwrap();
        }
        true
    });

    match result {
        Ok(value) => {
            assert_eq!(value.prefix, b"version-a");
            assert_eq!(value.verified.identity, identity(b"version-a"));
            assert_eq!(value.verified.size_bytes, 9);
        }
        Err(error) => assert_eq!(error.kind, ArtifactErrorKind::IdentityMismatch),
    }
    assert_eq!(std::fs::read(source).unwrap(), b"version-a");
}

fn identity(bytes: &[u8]) -> MediaIdentity {
    MediaIdentity {
        algorithm: HashAlgorithm::Sha256,
        digest: crate::ContentDigest::sha256(bytes).value,
    }
}

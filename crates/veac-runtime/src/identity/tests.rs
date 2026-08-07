use super::*;

#[test]
fn runtime_and_combined_identities_are_stable_and_exact() {
    assert_eq!(runtime_backend_identity(), runtime_backend_identity());
    assert_eq!(artifact_backend_identity(), artifact_backend_identity());
    runtime_backend_identity().validate().unwrap();
    artifact_backend_identity().validate().unwrap();
    let base = runtime_identity("source", "build");
    assert_ne!(runtime_identity("changed", "build"), base);
    assert_ne!(runtime_identity("source", "changed"), base);
    let codegen = ContentDigest::sha256(b"codegen");
    let runtime = ContentDigest::sha256(b"runtime");
    let combined = combined_identity(&codegen, &runtime);
    assert_ne!(
        combined_identity(&ContentDigest::sha256(b"other"), &runtime),
        combined
    );
    assert_ne!(
        combined_identity(&codegen, &ContentDigest::sha256(b"other")),
        combined
    );
}

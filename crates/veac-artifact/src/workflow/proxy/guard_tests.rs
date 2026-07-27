use std::cell::Cell;

use super::*;

#[test]
fn expired_guard_precedes_proxy_contract_validation() {
    let temp = tempfile::tempdir().unwrap();
    let calls = Cell::new(0);
    let error = select_proxy_while(
        &ArtifactStore::new(temp.path()),
        &ProxySelectionRequest {
            source_identity: ContentDigest::sha256(b"source"),
            video: None,
            audio: None,
        },
        || {
            calls.set(calls.get() + 1);
            false
        },
    )
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(calls.get(), 1);
}

use std::cell::Cell;

use super::*;

#[test]
fn expired_guard_precedes_render_segment_identity_work() {
    let plan = crate::test_support::plan(b"source");
    let mut contract = FullRenderSegmentContract::new(
        &plan,
        ContentDigest::sha256(b"clocks"),
        crate::test_support::producer(),
    )
    .unwrap();
    contract.descriptor.producer.version = "v".repeat(crate::MAX_ARTIFACT_JSON_STRING_BYTES);
    let temp = tempfile::tempdir().unwrap();
    let calls = Cell::new(0);
    let error =
        select_full_render_segment_while(&ArtifactStore::new(temp.path()), &contract, || {
            calls.set(calls.get() + 1);
            false
        })
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
    assert_eq!(calls.get(), 1);
}

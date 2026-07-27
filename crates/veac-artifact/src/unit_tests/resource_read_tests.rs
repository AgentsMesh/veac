use std::collections::BTreeMap;

use crate::{test_support, ArtifactErrorKind, ExecutionBindings};

#[test]
fn bound_resources_offer_default_and_tighter_bounded_verified_reads() {
    let bytes = b"verified resource";
    let plan = test_support::plan(bytes);
    let input = &plan.inputs[0];
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source");
    std::fs::write(&source, bytes).unwrap();
    let paths = BTreeMap::from([(input.id.clone(), source)]);
    let bindings = ExecutionBindings::from_originals(&plan, &paths).unwrap();
    let resource = bindings.input(&input.id).unwrap().resource().unwrap();

    assert_eq!(resource.read_verified().unwrap(), bytes);
    assert_eq!(
        resource.read_verified_bounded(bytes.len() as u64).unwrap(),
        bytes
    );
    let error = resource
        .read_verified_bounded(bytes.len() as u64 - 1)
        .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::ResourceLimit);
}

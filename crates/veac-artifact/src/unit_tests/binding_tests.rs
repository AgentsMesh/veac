use crate::{test_support::plan, *};

#[test]
fn binding_manifest_round_trips_verified_machine_paths() {
    let temp = tempfile::tempdir().unwrap();
    let media = temp.path().join("media.mp4");
    std::fs::write(&media, b"media").unwrap();
    let plan = plan(b"media");
    let bindings = crate::test_support::original_bindings(&plan, &media);
    let manifest = binding_manifest(&plan, &bindings).unwrap();
    assert_eq!(manifest.schema, EXECUTION_BINDING_SCHEMA_ID);
    assert!(manifest.inputs[0].path.is_absolute());
    let bytes = canonical_binding_bytes(&manifest).unwrap();
    let decoded: ExecutionBindingManifest = serde_json::from_slice(&bytes).unwrap();
    let restored = resolve_binding_manifest(&plan, &decoded).unwrap();
    assert_eq!(
        restored
            .input(&plan.inputs[0].id)
            .unwrap()
            .resource()
            .unwrap()
            .path(),
        media.canonicalize().unwrap().as_path()
    );
}

#[test]
fn binding_manifest_rejects_plan_identity_path_and_order_mismatches() {
    let temp = tempfile::tempdir().unwrap();
    let media = temp.path().join("media.mp4");
    std::fs::write(&media, b"media").unwrap();
    let plan = plan(b"media");
    let bindings = crate::test_support::original_bindings(&plan, &media);
    let manifest = binding_manifest(&plan, &bindings).unwrap();

    let mut wrong_plan = manifest.clone();
    wrong_plan.plan_hash = ContentDigest::sha256(b"other").value;
    assert!(resolve_binding_manifest(&plan, &wrong_plan).is_err());

    let mut wrong_identity = manifest.clone();
    wrong_identity.inputs[0].identity.digest = ContentDigest::sha256(b"wrong").value;
    assert!(resolve_binding_manifest(&plan, &wrong_identity).is_err());
    let mut unsupported = manifest.clone();
    unsupported.inputs[0].identity.algorithm = veac_ir::HashAlgorithm::Blake3;
    assert!(unsupported.validate().is_err());

    std::fs::write(&media, b"changed").unwrap();
    assert!(resolve_binding_manifest(&plan, &manifest).is_err());

    let mut empty_path = manifest;
    empty_path.inputs[0].path.clear();
    assert!(empty_path.validate().is_err());
}

#[test]
fn binding_manifest_requires_every_plan_input_and_valid_header() {
    let plan = plan(b"media");
    assert!(binding_manifest(&plan, &ExecutionBindings::default()).is_err());
    let invalid = ExecutionBindingManifest {
        schema: "wrong".to_owned(),
        schema_version: 1,
        plan_hash: ContentDigest::sha256(b"plan").value,
        inputs: vec![],
    };
    assert!(invalid.validate().is_err());
}

#[cfg(unix)]
#[test]
fn binding_manifest_rejects_symlink_inputs() {
    use std::os::unix::fs::symlink;

    let temp = tempfile::tempdir().unwrap();
    let media = temp.path().join("media.mp4");
    let linked = temp.path().join("linked.mp4");
    std::fs::write(&media, b"media").unwrap();
    symlink(&media, &linked).unwrap();
    let plan = plan(b"media");
    let bindings = crate::test_support::original_bindings(&plan, &linked);
    assert!(binding_manifest(&plan, &bindings).is_err());
}

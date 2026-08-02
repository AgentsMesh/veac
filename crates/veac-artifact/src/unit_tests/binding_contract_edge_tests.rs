use veac_plan::PlanInputId;

use crate::{test_support::plan, *};

#[test]
fn binding_manifest_requires_a_valid_hash_and_strict_input_order() {
    let temp = tempfile::tempdir().unwrap();
    let media = temp.path().join("media.mov");
    std::fs::write(&media, b"media").unwrap();
    let plan = plan(b"media");
    let bindings = crate::test_support::original_bindings(&plan, &media);
    let manifest = binding_manifest(&plan, &bindings).unwrap();

    let mut invalid_hash = manifest.clone();
    invalid_hash.plan_hash = "bad".into();
    assert_invalid(&invalid_hash);

    let mut duplicate = manifest;
    duplicate.inputs.push(duplicate.inputs[0].clone());
    assert_invalid(&duplicate);
}

#[test]
fn binding_resolution_rejects_count_and_input_id_mismatches() {
    let temp = tempfile::tempdir().unwrap();
    let media = temp.path().join("media.mov");
    std::fs::write(&media, b"media").unwrap();
    let plan = plan(b"media");
    let bindings = crate::test_support::original_bindings(&plan, &media);
    let manifest = binding_manifest(&plan, &bindings).unwrap();

    let mut missing = manifest.clone();
    missing.inputs.clear();
    assert_eq!(
        resolve_binding_manifest(&plan, &missing).unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );

    let mut wrong_id = manifest;
    wrong_id.inputs[0].input_id = PlanInputId::new("pin_other").unwrap();
    assert_eq!(
        resolve_binding_manifest(&plan, &wrong_id).unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

#[test]
fn binding_generation_rejects_non_file_paths() {
    let temp = tempfile::tempdir().unwrap();
    let plan = plan(b"media");
    let bindings = crate::test_support::original_bindings(&plan, temp.path());
    assert_eq!(
        binding_manifest(&plan, &bindings).unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
}

fn assert_invalid(manifest: &ExecutionBindingManifest) {
    assert_eq!(
        manifest.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

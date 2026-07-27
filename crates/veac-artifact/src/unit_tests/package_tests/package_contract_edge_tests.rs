use std::fs;

use crate::{test_support::*, *};

#[test]
fn package_rejects_an_invalid_project_and_snapshot_drift() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("source.mov");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let bindings = binding(&plan, &source);

    let mut invalid = project(b"media");
    invalid.schema = "invalid".into();
    let error = package_plan(&invalid, &plan, &bindings, &temp.path().join("invalid")).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::InvalidContract);
    assert!(std::error::Error::source(&error).is_some());

    assert_eq!(
        package_plan(
            &project(b"other"),
            &plan,
            &bindings,
            &temp.path().join("drift")
        )
        .unwrap_err()
        .kind,
        ArtifactErrorKind::IdentityMismatch
    );
}

#[test]
fn package_manifest_rejects_invalid_digests_and_project_media_collision() {
    let plan = plan(b"media");
    let base = PackageManifest {
        schema: PACKAGE_MANIFEST_SCHEMA_ID.into(),
        schema_version: 1,
        source: plan.header.source,
        plan_hash: ContentDigest::sha256(b"plan").value,
        project_path: "project.veac.json".into(),
        project_content: ContentDigest::sha256(b"project"),
        entries: vec![PackageEntry {
            input_id: plan.inputs[0].id.clone(),
            material_id: plan.inputs[0].material_id.clone(),
            identity: plan.inputs[0].observed_identity.clone(),
            packaged_path: "inputs/media".into(),
        }],
    };

    let mut invalid = base.clone();
    invalid.project_content.value = "bad".into();
    assert_invalid(&invalid);

    let mut invalid = base.clone();
    invalid.plan_hash = "bad".into();
    assert_invalid(&invalid);

    let mut invalid = base;
    invalid.project_path = invalid.entries[0].packaged_path.clone();
    assert_invalid(&invalid);
}

fn binding(plan: &veac_plan::ResolvedRenderPlan, path: &std::path::Path) -> ExecutionBindings {
    original_bindings(plan, path)
}

fn assert_invalid(value: &PackageManifest) {
    assert_eq!(
        value.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

use std::fs;

use veac_ir::HashAlgorithm;

use crate::{test_support::*, *};

#[test]
fn packaging_requires_every_binding_and_matching_content() {
    let root = tempfile::tempdir().unwrap();
    let plan = plan(b"media");
    let missing = ExecutionBindings::default();
    assert_kind(
        package(&plan, &missing, &root.path().join("missing")),
        ArtifactErrorKind::MissingBinding,
    );
    let wrong = root.path().join("wrong.mp4");
    fs::write(&wrong, b"wrong").unwrap();
    let bindings = original_bindings(&plan, &wrong);
    assert_kind(
        package(&plan, &bindings, &root.path().join("wrong")),
        ArtifactErrorKind::IdentityMismatch,
    );

    let source = root.path().join("source.mp4");
    fs::write(&source, b"media").unwrap();
    let file_destination = root.path().join("package-file");
    fs::write(&file_destination, b"not a directory").unwrap();
    assert_kind(
        package(&plan, &binding(&plan, &source), &file_destination),
        ArtifactErrorKind::Io,
    );
    let mut invalid_plan = crate::test_support::plan(b"media");
    invalid_plan.sequences[0].tracks[0].clips[0]
        .visual
        .as_mut()
        .unwrap()
        .opacity = veac_ir::Animatable::constant(f64::NAN);
    let error = package(
        &invalid_plan,
        &binding(&invalid_plan, &source),
        &root.path().join("serialization"),
    )
    .unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::Serialization);
    assert!(std::error::Error::source(&error).is_some());
}

#[test]
fn unsupported_and_malformed_identities_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::write(&source, b"media").unwrap();
    for digest in ["bad".to_owned(), "A".repeat(64)] {
        let mut plan = plan(b"media");
        plan.inputs[0].observed_identity.digest = digest;
        assert_kind(
            package(
                &plan,
                &binding(&plan, &source),
                &root.path().join("invalid"),
            ),
            ArtifactErrorKind::InvalidContract,
        );
    }
    let mut plan = plan(b"media");
    plan.inputs[0].observed_identity.algorithm = HashAlgorithm::Blake3;
    assert_kind(
        package(
            &plan,
            &binding(&plan, &source),
            &root.path().join("unsupported"),
        ),
        ArtifactErrorKind::UnsupportedIdentity,
    );
}

#[test]
fn package_contract_rejects_headers_order_and_unsafe_paths() {
    let plan = plan(b"media");
    let base = PackageManifest {
        schema: PACKAGE_MANIFEST_SCHEMA_ID.to_owned(),
        schema_version: 1,
        source: plan.header.source,
        plan_hash: ContentDigest::sha256(b"plan").value,
        project_path: "project.veac.json".to_owned(),
        project_content: ContentDigest::sha256(b"project"),
        entries: vec![PackageEntry {
            input_id: plan.inputs[0].id.clone(),
            material_id: plan.inputs[0].material_id.clone(),
            identity: plan.inputs[0].observed_identity.clone(),
            packaged_path: "inputs/sha256/value".to_owned(),
        }],
    };
    let mut invalid = base.clone();
    invalid.schema = "other".to_owned();
    assert_invalid(&invalid);
    invalid = base.clone();
    invalid.schema_version = 2;
    assert_invalid(&invalid);
    invalid = base.clone();
    invalid.entries.push(invalid.entries[0].clone());
    assert_invalid(&invalid);
    for path in ["", "../media", "/media", "a\\b", "C:media"] {
        let mut invalid = base.clone();
        invalid.entries[0].packaged_path = path.to_owned();
        assert_eq!(
            invalid.validate().unwrap_err().kind,
            ArtifactErrorKind::UnsafePath
        );
    }
    let mut invalid = base;
    invalid.project_path = "../project.json".to_owned();
    assert_eq!(
        invalid.validate().unwrap_err().kind,
        ArtifactErrorKind::UnsafePath
    );
    invalid.project_path = "package.json".to_owned();
    assert_invalid(&invalid);
}

#[test]
fn existing_corrupt_package_input_and_missing_restore_input_fail() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let destination = root.path().join("package");
    let expected = destination
        .join("inputs/sha256")
        .join(&plan.inputs[0].observed_identity.digest);
    fs::create_dir_all(expected.parent().unwrap()).unwrap();
    fs::write(&expected, b"corrupt").unwrap();
    assert_kind(
        package(&plan, &binding(&plan, &source), &destination),
        ArtifactErrorKind::IdentityMismatch,
    );
    fs::remove_file(expected).unwrap();
    let manifest = package(&plan, &binding(&plan, &source), &destination).unwrap();
    fs::remove_file(destination.join(&manifest.entries[0].packaged_path)).unwrap();
    assert_kind(
        package_binding_manifest(&destination, &manifest),
        ArtifactErrorKind::Io,
    );
    fs::write(destination.join(&manifest.project_path), b"corrupt").unwrap();
    assert_kind(
        packaged_project(&destination, &manifest),
        ArtifactErrorKind::IdentityMismatch,
    );
}

fn binding(plan: &veac_plan::ResolvedRenderPlan, path: &std::path::Path) -> ExecutionBindings {
    original_bindings(plan, path)
}

fn package(
    plan: &veac_plan::ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    destination: &std::path::Path,
) -> ArtifactResult<PackageManifest> {
    package_plan(&project(b"media"), plan, bindings, destination)
}

fn assert_invalid(value: &PackageManifest) {
    assert_eq!(
        value.validate().unwrap_err().kind,
        ArtifactErrorKind::InvalidContract
    );
}

fn assert_kind<T: std::fmt::Debug>(result: ArtifactResult<T>, kind: ArtifactErrorKind) {
    assert_eq!(result.unwrap_err().kind, kind);
}

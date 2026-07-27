#![cfg(unix)]

use std::{fs, os::unix::fs::symlink};

use crate::{test_support::*, *};

#[test]
fn package_rejects_symlinked_internal_directory() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let package = root.path().join("package");
    let outside = root.path().join("outside");
    fs::create_dir(&package).unwrap();
    fs::create_dir(&outside).unwrap();
    symlink(&outside, package.join("inputs")).unwrap();

    let error = build_package(&plan, &binding(&plan, &source), &package).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}

#[test]
fn package_rejects_symlinked_root_without_writing_through_it() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let outside = root.path().join("outside");
    let package = root.path().join("package");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, &package).unwrap();

    let error = build_package(&plan, &binding(&plan, &source), &package).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(fs::read_dir(outside).unwrap().count(), 0);
}

#[test]
fn package_rejects_symlinked_digest_destination_without_touching_victim() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let package = root.path().join("package");
    let digest = &plan.inputs[0].observed_identity.digest;
    let destination = package.join("inputs/sha256").join(digest);
    let victim = root.path().join("victim");
    fs::create_dir_all(destination.parent().unwrap()).unwrap();
    fs::write(&victim, b"protected").unwrap();
    symlink(&victim, &destination).unwrap();

    let error = build_package(&plan, &binding(&plan, &source), &package).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(fs::read(victim).unwrap(), b"protected");
}

#[test]
fn package_manifest_symlink_cannot_truncate_an_external_file() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let package = root.path().join("package");
    let victim = root.path().join("victim");
    fs::create_dir(&package).unwrap();
    fs::write(&victim, b"protected").unwrap();
    symlink(&victim, package.join("package.json")).unwrap();

    let error = build_package(&plan, &binding(&plan, &source), &package).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
    assert_eq!(fs::read(victim).unwrap(), b"protected");
}

#[test]
fn package_manifest_ignores_predictable_temporary_symlink() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let package = root.path().join("package");
    let victim = root.path().join("victim");
    fs::create_dir(&package).unwrap();
    fs::write(&victim, b"protected").unwrap();
    let temporary = package.join(format!("package.json.{}.tmp", std::process::id()));
    symlink(&victim, &temporary).unwrap();

    build_package(&plan, &binding(&plan, &source), &package).unwrap();
    assert_eq!(fs::read(victim).unwrap(), b"protected");
    assert!(package.join("package.json").is_file());
    assert!(fs::symlink_metadata(temporary)
        .unwrap()
        .file_type()
        .is_symlink());
}

#[test]
fn package_binding_manifest_rejects_symlinked_payload() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let package = root.path().join("package");
    let manifest = build_package(&plan, &binding(&plan, &source), &package).unwrap();
    let input = package.join(&manifest.entries[0].packaged_path);
    let outside = root.path().join("outside");
    fs::write(&outside, b"media").unwrap();
    fs::remove_file(&input).unwrap();
    symlink(&outside, &input).unwrap();

    let error = package_binding_manifest(&package, &manifest).unwrap_err();
    assert_eq!(error.kind, ArtifactErrorKind::UnsafePath);
}

#[test]
fn package_supports_a_new_nested_destination() {
    let root = tempfile::tempdir().unwrap();
    let source = root.path().join("source");
    fs::write(&source, b"media").unwrap();
    let plan = plan(b"media");
    let package = root.path().join("new/nested/package");

    let manifest = build_package(&plan, &binding(&plan, &source), &package).unwrap();
    let bindings = package_binding_manifest(&package, &manifest).unwrap();
    assert_eq!(bindings.inputs.len(), 1);
}

fn binding(plan: &veac_plan::ResolvedRenderPlan, path: &std::path::Path) -> ExecutionBindings {
    original_bindings(plan, path)
}

fn build_package(
    plan: &veac_plan::ResolvedRenderPlan,
    bindings: &ExecutionBindings,
    destination: &std::path::Path,
) -> ArtifactResult<PackageManifest> {
    package_plan(&project(b"media"), plan, bindings, destination)
}

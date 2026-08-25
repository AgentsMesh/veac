use std::path::{Path, PathBuf};

use tempfile::{tempdir, TempDir};

use super::support::{canonical_project, pin_first_material, FakeEnvironment, MEDIA_SOURCE};
use crate::arguments::{PackageBindingsArgs, RelinkArgs};

#[test]
fn package_bindings_restore_verified_package_inputs() {
    let fixture = PackageFixture::new();
    let output = fixture.temp.path().join("bindings.json");

    crate::commands::package_bindings(PackageBindingsArgs {
        package: fixture.package.clone(),
        output: Some(output.clone()),
    })
    .unwrap();

    let manifest: veac_artifact::ExecutionBindingManifest =
        serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(manifest.inputs.len(), 1);
    assert!(manifest.inputs[0]
        .path
        .starts_with(fixture.package.canonicalize().unwrap()));
}

#[test]
fn package_bindings_reject_output_inside_package_and_unsafe_roots() {
    let fixture = PackageFixture::new();
    let error = package_bindings(
        &fixture.package,
        Some(fixture.package.join("bindings.json")),
    )
    .unwrap_err();
    assert!(error
        .to_string()
        .contains("PACKAGE_BINDINGS_OUTPUT_CONFLICT"));

    let file = fixture.temp.path().join("not-a-package");
    std::fs::write(&file, b"file").unwrap();
    assert!(package_bindings(&file, None)
        .unwrap_err()
        .to_string()
        .contains("non-symlink directory"));
    assert!(package_bindings(&fixture.temp.path().join("missing"), None)
        .unwrap_err()
        .to_string()
        .contains("PACKAGE_BINDINGS_FAILED"));
}

#[cfg(unix)]
#[test]
fn package_bindings_reject_a_symlinked_root() {
    let fixture = PackageFixture::new();
    let link = fixture.temp.path().join("package-link");
    std::os::unix::fs::symlink(&fixture.package, &link).unwrap();
    assert!(package_bindings(&link, None)
        .unwrap_err()
        .to_string()
        .contains("non-symlink directory"));
}

#[test]
fn package_bindings_maps_output_and_package_integrity_failures() {
    let fixture = PackageFixture::new();
    let missing_parent = fixture.temp.path().join("missing/output.json");
    assert!(package_bindings(&fixture.package, Some(missing_parent))
        .unwrap_err()
        .to_string()
        .contains("PACKAGE_BINDINGS_FAILED"));

    let corrupt_project = PackageFixture::new();
    std::fs::write(
        corrupt_project.package.join("project.veac.json"),
        b"corrupt",
    )
    .unwrap();
    assert!(package_bindings(&corrupt_project.package, None)
        .unwrap_err()
        .to_string()
        .contains("PACKAGE_BINDINGS_FAILED"));

    let corrupt_payload = PackageFixture::new();
    std::fs::write(corrupt_payload.packaged_input(), b"corrupt").unwrap();
    assert!(package_bindings(&corrupt_payload.package, None)
        .unwrap_err()
        .to_string()
        .contains("PACKAGE_BINDINGS_FAILED"));
}

#[test]
fn relink_emits_exact_bindings_and_reports_incomplete_searches() {
    let fixture = PackageFixture::new();
    let search = fixture.temp.path().join("search");
    std::fs::create_dir(&search).unwrap();
    let replacement = search.join("replacement.mp4");
    std::fs::copy(fixture.packaged_input(), &replacement).unwrap();
    let output = fixture.temp.path().join("relinked.json");

    relink(&fixture, &search, Some(output.clone())).unwrap();
    let manifest: veac_artifact::ExecutionBindingManifest =
        serde_json::from_slice(&std::fs::read(output).unwrap()).unwrap();
    assert_eq!(manifest.inputs[0].path, replacement.canonicalize().unwrap());

    std::fs::copy(fixture.packaged_input(), search.join("duplicate.mp4")).unwrap();
    let ambiguous = relink(&fixture, &search, None).unwrap_err().to_string();
    assert!(ambiguous.contains("ambiguous inputs: ["));
    assert!(!ambiguous.contains("ambiguous inputs: []"));

    let empty = fixture.temp.path().join("empty-search");
    std::fs::create_dir(&empty).unwrap();
    let unresolved = relink(&fixture, &empty, None).unwrap_err().to_string();
    assert!(unresolved.contains("unresolved inputs: ["));
    assert!(!unresolved.contains("unresolved inputs: []"));
}

#[test]
fn relink_maps_discovery_failures_and_protects_candidates() {
    let fixture = PackageFixture::new();
    let missing = fixture.temp.path().join("missing-search");
    assert!(relink(&fixture, &missing, None)
        .unwrap_err()
        .to_string()
        .contains("RELINK_FAILED"));

    let search = fixture.temp.path().join("protected-search");
    std::fs::create_dir(&search).unwrap();
    let replacement = search.join("replacement.mp4");
    std::fs::copy(fixture.packaged_input(), &replacement).unwrap();
    assert!(relink(&fixture, &search, Some(replacement))
        .unwrap_err()
        .to_string()
        .contains("OUTPUT_OVERWRITES_INPUT"));
}

fn package_bindings(package: &Path, output: Option<PathBuf>) -> crate::CliResult {
    crate::commands::package_bindings(PackageBindingsArgs {
        package: package.to_owned(),
        output,
    })
}

fn relink(fixture: &PackageFixture, search: &Path, output: Option<PathBuf>) -> crate::CliResult {
    crate::commands::relink(RelinkArgs {
        project: fixture.package.join("project.veac.json"),
        config: None,
        search: vec![search.to_owned()],
        output,
    })
}

struct PackageFixture {
    temp: TempDir,
    package: PathBuf,
    manifest: veac_artifact::PackageManifest,
}

impl PackageFixture {
    fn new() -> Self {
        let temp = tempdir().unwrap();
        let media = temp.path().join("clip.mp4");
        std::fs::write(&media, b"portable media").unwrap();
        let project = canonical_project(&temp, MEDIA_SOURCE);
        let mut environment = FakeEnvironment::success();
        environment.observed = veac_runtime::asset::sha256_identity(&media).unwrap();
        pin_first_material(&project, environment.observed.clone());
        let package = temp.path().join("package");
        crate::commands::bundle(&project, None, None, None, &package, &environment).unwrap();
        let manifest =
            serde_json::from_slice(&std::fs::read(package.join("package.json")).unwrap()).unwrap();
        Self {
            temp,
            package,
            manifest,
        }
    }

    fn packaged_input(&self) -> PathBuf {
        self.package.join(&self.manifest.entries[0].packaged_path)
    }
}

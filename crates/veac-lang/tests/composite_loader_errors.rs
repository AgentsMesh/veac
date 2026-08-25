use std::path::Path;

use veac_lang::package::{ExactVersion, PackageIdentity, PackageName, PackageSourceLoader};
use veac_lang::program::{CompositeSourceLoader, FileSystemLoader, SourceLoader};

#[path = "package_contract_helpers.rs"]
#[allow(dead_code)]
mod helpers;

fn identity(name: &str, version: &str) -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new(name).unwrap(),
        version: ExactVersion::new(version).unwrap(),
    }
}

fn project(root: &Path) -> CompositeSourceLoader {
    std::fs::write(root.join("main.veac"), "project { output = 1s }\n").unwrap();
    std::fs::write(root.join("local.veac"), "module {}\n").unwrap();
    let (loader, _) = FileSystemLoader::for_entry(&root.join("main.veac")).unwrap();
    CompositeSourceLoader::new(loader)
}

fn package(root: &Path) -> (PackageIdentity, PackageSourceLoader) {
    helpers::write_contract(root);
    let (loader, _) = PackageSourceLoader::for_root(root).unwrap();
    (identity("root", "1.0.0"), loader)
}

#[test]
fn ordinary_imports_fall_back_to_the_project_loader() {
    let root = tempfile::tempdir().unwrap();
    let loader = project(root.path());
    let source = loader.load("main.veac", "./local.veac").unwrap();
    assert_eq!(source.id, "local.veac");
    assert_eq!(source.source, "module {}\n");
}

#[test]
fn duplicate_package_mount_is_rejected() {
    let project_root = tempfile::tempdir().unwrap();
    let first_root = tempfile::tempdir().unwrap();
    let second_root = tempfile::tempdir().unwrap();
    let mut loader = project(project_root.path());
    let (package_id, first) = package(first_root.path());
    helpers::write_contract(second_root.path());
    let second_source = b"module { }\n";
    std::fs::write(second_root.path().join("main.veac"), second_source).unwrap();
    helpers::rewrite_entry_digest(second_root.path(), second_source);
    let (second, second_entry) = PackageSourceLoader::for_root(second_root.path()).unwrap();
    assert_ne!(first.load_entry().unwrap().source, second_entry.source);

    loader.mount_package(package_id.clone(), first).unwrap();
    let error = loader
        .mount_package(package_id.clone(), second)
        .unwrap_err();
    assert_eq!(error, "package identity is already mounted");
    assert_eq!(
        loader.load_package_entry(&package_id).unwrap().source,
        "module {}\n"
    );
}

#[test]
fn every_unmounted_package_route_fails_with_its_exact_selector() {
    let root = tempfile::tempdir().unwrap();
    let loader = project(root.path());
    let missing = identity("missing", "2.3.4");

    let entry = loader.load_package_entry(&missing).unwrap_err();
    assert!(entry.contains("missing@2.3.4 is not mounted"));
    for (importer, requested) in [
        ("main.veac", "package:missing@2.3.4/main.veac"),
        ("packages/missing@2.3.4/main.veac", "./local.veac"),
        (
            "packages/missing@2.3.4/main.veac",
            "package:other@1.0.0/lib.veac",
        ),
    ] {
        let error = loader.load(importer, requested).unwrap_err();
        assert!(error.contains("missing@2.3.4 is not mounted"), "{error}");
    }
}

#[test]
fn package_read_failures_preserve_the_package_contract_error() {
    let project_root = tempfile::tempdir().unwrap();
    let package_root = tempfile::tempdir().unwrap();
    let mut loader = project(project_root.path());
    let (package_id, package) = package(package_root.path());
    loader.mount_package(package_id.clone(), package).unwrap();

    let relative = loader
        .load("packages/root@1.0.0/main.veac", "./missing.veac")
        .unwrap_err();
    assert!(relative.contains("not declared"), "{relative}");
    let qualified = loader
        .load("main.veac", "package:root@1.0.0/missing.veac")
        .unwrap_err();
    assert!(qualified.contains("not declared"), "{qualified}");

    std::fs::write(package_root.path().join("main.veac"), "module { }\n").unwrap();
    let entry = loader.load_package_entry(&package_id).unwrap_err();
    assert!(entry.contains("digest mismatch"), "{entry}");
}

#[test]
fn malformed_package_importers_and_requests_fail_closed() {
    let root = tempfile::tempdir().unwrap();
    let loader = project(root.path());
    for (importer, requested) in [
        ("packages/root@1.0.0", "./module.veac"),
        ("packages/root/main.veac", "./module.veac"),
        ("main.veac", "package:@1.0.0/module.veac"),
        ("main.veac", "package:root@1.0.0/a\0b.veac"),
        ("main.veac", "package:root@1.0.0/a\nb.veac"),
        ("main.veac", "package:root@1.0.0/a/./b.veac"),
    ] {
        assert!(
            loader.load(importer, requested).is_err(),
            "accepted {requested:?}"
        );
    }
}

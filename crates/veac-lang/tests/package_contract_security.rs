use veac_lang::package::api::{canonical_api_metadata_json, package_api_digest, ApiMetadataV1};
use veac_lang::package::{
    canonical_package_lock_json, package_content_digest, parse_package_lock_json, sha256_bytes,
    ExactVersion, LockedFile, LockedPackage, PackageDependency, PackageIdentity, PackageName,
    PackageSourceLoader,
};
use veac_lang::program::SourceLoader;

#[path = "package_contract_helpers.rs"]
#[allow(dead_code)]
mod helpers;

#[test]
fn root_local_modules_must_be_locked_and_are_part_of_content_identity() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    std::fs::write(temp.path().join("helper.veac"), "module {}\n").unwrap();
    let (loader, _) = PackageSourceLoader::for_root(temp.path()).unwrap();
    assert!(loader
        .load("packages/root@1.0.0/main.veac", "./helper.veac")
        .is_err());

    lock_root_source(temp.path(), "helper.veac", b"module {}\n");
    let first = veac_lang::package::discover_package(temp.path()).unwrap();
    let (loader, _) = PackageSourceLoader::for_root(temp.path()).unwrap();
    assert_eq!(
        loader
            .load("packages/root@1.0.0/main.veac", "./helper.veac")
            .unwrap()
            .id,
        "packages/root@1.0.0/helper.veac"
    );

    std::fs::write(temp.path().join("helper.veac"), "module { }\n").unwrap();
    assert!(veac_lang::package::discover_package(temp.path()).is_err());
    assert_ne!(
        first.root.content_sha256,
        package_content_digest(&[LockedFile {
            path: "main.veac".to_owned(),
            sha256: sha256_bytes(b"module {}\n"),
        }])
        .unwrap()
    );
}

#[test]
fn imports_may_cross_only_direct_dependency_edges() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    add_dependency(temp.path(), "other", "3.0.0", "vendor/other");
    let (loader, _) = PackageSourceLoader::for_root(temp.path()).unwrap();
    let error = loader
        .load(
            "packages/util@2.0.0/lib.veac",
            "package:other@3.0.0/lib.veac",
        )
        .unwrap_err();
    assert!(error.contains("undeclared package dependency edge"));
    assert!(loader
        .load(
            "packages/root@1.0.0/main.veac",
            "package:other@3.0.0/lib.veac",
        )
        .is_ok());
}

#[test]
fn package_local_resolution_precedes_a_same_named_root_source() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    std::fs::write(temp.path().join("helper.veac"), "module {}\n").unwrap();
    std::fs::write(
        temp.path().join("vendor/util/helper.veac"),
        "module { export fn local() -> time { 1s } }\n",
    )
    .unwrap();
    lock_root_source(temp.path(), "helper.veac", b"module {}\n");
    helpers::lock_dependency_source(
        temp.path(),
        "helper.veac",
        b"module { export fn local() -> time { 1s } }\n",
    );
    let (loader, _) = PackageSourceLoader::for_root(temp.path()).unwrap();
    assert_eq!(
        loader
            .load("packages/util@2.0.0/lib.veac", "helper.veac")
            .unwrap()
            .id,
        "packages/util@2.0.0/helper.veac"
    );
}

#[test]
fn root_sources_cannot_claim_dependency_source_ids() {
    let temp = tempfile::tempdir().unwrap();
    helpers::write_contract(temp.path());
    let text = std::fs::read_to_string(temp.path().join("veac.package.lock")).unwrap();
    let mut lock = parse_package_lock_json(&text).unwrap();
    lock.root_files.push(LockedFile {
        path: "vendor/util/lib.veac".to_owned(),
        sha256: sha256_bytes(b"module {}\n"),
    });
    lock.root_files
        .sort_by(|left, right| left.path.cmp(&right.path));
    lock.root_content_sha256 = package_content_digest(&lock.root_files).unwrap();
    assert!(lock.validate().is_err());
}

fn lock_root_source(root: &std::path::Path, path: &str, source: &[u8]) {
    let lock_path = root.join("veac.package.lock");
    let mut lock = parse_package_lock_json(&std::fs::read_to_string(&lock_path).unwrap()).unwrap();
    lock.root_files.push(LockedFile {
        path: path.to_owned(),
        sha256: sha256_bytes(source),
    });
    lock.root_files
        .sort_by(|left, right| left.path.cmp(&right.path));
    lock.root_content_sha256 = package_content_digest(&lock.root_files).unwrap();
    std::fs::write(lock_path, canonical_package_lock_json(&lock).unwrap()).unwrap();
}

fn add_dependency(root: &std::path::Path, name: &str, version: &str, path: &str) {
    let package = identity(name, version);
    let source = b"module {}\n";
    let file = LockedFile {
        path: "lib.veac".to_owned(),
        sha256: sha256_bytes(source),
    };
    let api = ApiMetadataV1::new(package.clone(), vec![]);
    let locked = LockedPackage {
        package: package.clone(),
        path: path.to_owned(),
        entry: "lib.veac".to_owned(),
        dependencies: vec![],
        files: vec![file.clone()],
        content_sha256: package_content_digest(&[file]).unwrap(),
        api_sha256: package_api_digest(&api).unwrap(),
    };
    std::fs::create_dir_all(root.join(path)).unwrap();
    std::fs::write(root.join(path).join("lib.veac"), source).unwrap();
    std::fs::write(
        root.join(path).join("veac.package.api.json"),
        canonical_api_metadata_json(&api).unwrap(),
    )
    .unwrap();
    let lock_path = root.join("veac.package.lock");
    let mut lock = parse_package_lock_json(&std::fs::read_to_string(&lock_path).unwrap()).unwrap();
    lock.packages.push(locked);
    lock.packages
        .sort_by(|left, right| left.package.cmp(&right.package));
    rewrite_root_dependencies(root, &package);
    std::fs::write(lock_path, canonical_package_lock_json(&lock).unwrap()).unwrap();
}

fn rewrite_root_dependencies(root: &std::path::Path, package: &PackageIdentity) {
    use veac_lang::package::{canonical_package_manifest_json, parse_package_manifest_json};
    let path = root.join("veac.package.json");
    let mut manifest =
        parse_package_manifest_json(&std::fs::read_to_string(&path).unwrap()).unwrap();
    manifest.dependencies.push(PackageDependency {
        name: package.name.clone(),
        version: package.version.clone(),
    });
    manifest
        .dependencies
        .sort_by_key(PackageDependency::identity);
    std::fs::write(path, canonical_package_manifest_json(&manifest).unwrap()).unwrap();
}

fn identity(name: &str, version: &str) -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new(name).unwrap(),
        version: ExactVersion::new(version).unwrap(),
    }
}

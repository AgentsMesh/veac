use std::path::Path;

use crate::package::api::{canonical_api_metadata_json, package_api_digest, ApiMetadataV1};
use crate::package::{
    canonical_package_lock_json, canonical_package_manifest_json, package_content_digest,
    sha256_bytes, ExactVersion, LockedFile, LockedPackage, PackageDependency, PackageIdentity,
    PackageLockV1, PackageManifestV1, PackageName,
};

pub(super) fn write_package(root: &Path, name: &str, source: &str) {
    let package = identity(name);
    let api = ApiMetadataV1::new(package.clone(), vec![]);
    let file = locked_file("main.veac", source);
    let manifest = PackageManifestV1::new(
        package.clone(),
        "main.veac",
        file.sha256.clone(),
        Vec::new(),
        package_api_digest(&api).unwrap(),
    );
    let lock = PackageLockV1::new(package, vec![file], Vec::new()).unwrap();
    write_contract(root, source, &api, &manifest, &lock);
}

pub(super) fn write_with_shared(root: &Path, name: &str, shared_source: &str) {
    let package = identity(name);
    let shared = identity("shared");
    let dependency = PackageDependency {
        name: shared.name.clone(),
        version: shared.version.clone(),
    };
    let root_source = "module {}\n";
    let root_api = ApiMetadataV1::new(package.clone(), vec![]);
    let shared_api = ApiMetadataV1::new(shared.clone(), vec![]);
    let root_file = locked_file("main.veac", root_source);
    let shared_file = locked_file("main.veac", shared_source);
    let locked = LockedPackage {
        package: shared,
        path: "vendor/shared".into(),
        entry: "main.veac".into(),
        dependencies: Vec::new(),
        files: vec![shared_file.clone()],
        content_sha256: package_content_digest(std::slice::from_ref(&shared_file)).unwrap(),
        api_sha256: package_api_digest(&shared_api).unwrap(),
    };
    let manifest = PackageManifestV1::new(
        package.clone(),
        "main.veac",
        root_file.sha256.clone(),
        vec![dependency],
        package_api_digest(&root_api).unwrap(),
    );
    let lock = PackageLockV1::new(package, vec![root_file], vec![locked]).unwrap();
    write_contract(root, root_source, &root_api, &manifest, &lock);
    let shared_root = root.join("vendor/shared");
    std::fs::create_dir_all(&shared_root).unwrap();
    std::fs::write(shared_root.join("main.veac"), shared_source).unwrap();
    write(
        &shared_root.join("veac.package.api.json"),
        canonical_api_metadata_json(&shared_api).unwrap(),
    );
}

fn write_contract(
    root: &Path,
    source: &str,
    api: &ApiMetadataV1,
    manifest: &PackageManifestV1,
    lock: &PackageLockV1,
) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("main.veac"), source).unwrap();
    write(
        &root.join("veac.package.api.json"),
        canonical_api_metadata_json(api).unwrap(),
    );
    write(
        &root.join("veac.package.json"),
        canonical_package_manifest_json(manifest).unwrap(),
    );
    write(
        &root.join("veac.package.lock"),
        canonical_package_lock_json(lock).unwrap(),
    );
}

pub(super) fn identity(name: &str) -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new(name).unwrap(),
        version: ExactVersion::new("1.0.0").unwrap(),
    }
}

fn locked_file(path: &str, source: &str) -> LockedFile {
    LockedFile {
        path: path.into(),
        sha256: sha256_bytes(source.as_bytes()),
    }
}

fn write(path: &Path, value: String) {
    std::fs::write(path, format!("{value}\n")).unwrap();
}

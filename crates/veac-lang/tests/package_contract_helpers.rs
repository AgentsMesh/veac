use std::path::Path;

use veac_lang::package::api::{
    canonical_api_metadata_json, from_module_interface, package_api_digest, ApiMetadataV1,
};
use veac_lang::package::{
    canonical_package_lock_json, canonical_package_manifest_json, package_content_digest,
    parse_package_lock_json, parse_package_manifest_json, sha256_bytes, ExactVersion, LockedFile,
    LockedPackage, PackageDependency, PackageIdentity, PackageLockV1, PackageManifestV1,
    PackageName,
};
use veac_lang::program::CompilerDatabase;

pub fn write_contract(root: &Path) {
    let root_identity = identity("root", "1.0.0");
    let dependency_identity = identity("util", "2.0.0");
    let dependency_source = b"module {}\n";
    let dependency_api = ApiMetadataV1::new(dependency_identity.clone(), vec![]);
    let dependency_file = LockedFile {
        path: "lib.veac".to_owned(),
        sha256: sha256_bytes(dependency_source),
    };
    let dependency = LockedPackage {
        package: dependency_identity.clone(),
        path: "vendor/util".to_owned(),
        entry: "lib.veac".to_owned(),
        dependencies: vec![],
        files: vec![dependency_file.clone()],
        content_sha256: package_content_digest(&[dependency_file]).unwrap(),
        api_sha256: package_api_digest(&dependency_api).unwrap(),
    };
    let root_api = ApiMetadataV1::new(root_identity.clone(), vec![]);
    let root_file = LockedFile {
        path: "main.veac".to_owned(),
        sha256: sha256_bytes(b"module {}\n"),
    };
    let root_manifest = PackageManifestV1::new(
        root_identity.clone(),
        "main.veac",
        sha256_bytes(b"module {}\n"),
        vec![PackageDependency {
            name: dependency_identity.name.clone(),
            version: dependency_identity.version.clone(),
        }],
        package_api_digest(&root_api).unwrap(),
    );
    let lock = PackageLockV1::new(root_identity, vec![root_file], vec![dependency]).unwrap();
    std::fs::create_dir_all(root.join("vendor/util")).unwrap();
    std::fs::write(root.join("main.veac"), "module {}\n").unwrap();
    std::fs::write(root.join("vendor/util/lib.veac"), dependency_source).unwrap();
    std::fs::write(
        root.join("veac.package.api.json"),
        canonical_api_metadata_json(&root_api).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("vendor/util/veac.package.api.json"),
        canonical_api_metadata_json(&dependency_api).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("veac.package.json"),
        canonical_package_manifest_json(&root_manifest).unwrap(),
    )
    .unwrap();
    std::fs::write(
        root.join("veac.package.lock"),
        canonical_package_lock_json(&lock).unwrap(),
    )
    .unwrap();
}

fn identity(name: &str, version: &str) -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new(name).unwrap(),
        version: ExactVersion::new(version).unwrap(),
    }
}

pub fn rewrite_dependency_digest(root: &Path, source: &[u8]) {
    let text = std::fs::read_to_string(root.join("veac.package.lock")).unwrap();
    let mut lock: PackageLockV1 = parse_package_lock_json(&text).unwrap();
    lock.packages[0].files[0].sha256 = sha256_bytes(source);
    lock.packages[0].content_sha256 = package_content_digest(&lock.packages[0].files).unwrap();
    std::fs::write(
        root.join("veac.package.lock"),
        canonical_package_lock_json(&lock).unwrap(),
    )
    .unwrap();
}

pub fn rewrite_entry_digest(root: &Path, source: &[u8]) {
    let text = std::fs::read_to_string(root.join("veac.package.json")).unwrap();
    let mut manifest = parse_package_manifest_json(&text).unwrap();
    manifest.entry_sha256 = sha256_bytes(source);
    std::fs::write(
        root.join("veac.package.json"),
        canonical_package_manifest_json(&manifest).unwrap(),
    )
    .unwrap();
    let text = std::fs::read_to_string(root.join("veac.package.lock")).unwrap();
    let mut lock = parse_package_lock_json(&text).unwrap();
    let entry = lock
        .root_files
        .iter_mut()
        .find(|file| file.path == manifest.entry)
        .unwrap();
    entry.sha256 = sha256_bytes(source);
    lock.root_content_sha256 = package_content_digest(&lock.root_files).unwrap();
    std::fs::write(
        root.join("veac.package.lock"),
        canonical_package_lock_json(&lock).unwrap(),
    )
    .unwrap();
}

pub fn rewrite_lock_root(root: &Path, identity: PackageIdentity) {
    let text = std::fs::read_to_string(root.join("veac.package.lock")).unwrap();
    let mut lock = parse_package_lock_json(&text).unwrap();
    lock.root = identity;
    std::fs::write(
        root.join("veac.package.lock"),
        canonical_package_lock_json(&lock).unwrap(),
    )
    .unwrap();
}

pub fn lock_dependency_source(root: &Path, path: &str, source: &[u8]) {
    let text = std::fs::read_to_string(root.join("veac.package.lock")).unwrap();
    let mut lock = parse_package_lock_json(&text).unwrap();
    lock.packages[0].files.push(LockedFile {
        path: path.to_owned(),
        sha256: sha256_bytes(source),
    });
    lock.packages[0]
        .files
        .sort_by(|left, right| left.path.cmp(&right.path));
    lock.packages[0].content_sha256 = package_content_digest(&lock.packages[0].files).unwrap();
    std::fs::write(
        root.join("veac.package.lock"),
        canonical_package_lock_json(&lock).unwrap(),
    )
    .unwrap();
}

pub fn derive_root_api(root: &Path) -> ApiMetadataV1 {
    let entry = root.join("main.veac");
    let (loader, entry) = veac_lang::program::FileSystemLoader::for_entry(&entry).unwrap();
    let interface = CompilerDatabase::default()
        .module_interface(entry, &loader)
        .unwrap();
    from_module_interface(identity("root", "1.0.0"), &interface)
}

pub fn derive_api(entry: std::path::PathBuf, package: PackageIdentity) -> ApiMetadataV1 {
    let (loader, source) = veac_lang::program::FileSystemLoader::for_entry(&entry).unwrap();
    let interface = CompilerDatabase::default()
        .module_interface(source, &loader)
        .unwrap();
    from_module_interface(package, &interface)
}

pub fn rewrite_dependency_api(root: &Path, api: &ApiMetadataV1) {
    std::fs::write(
        root.join("vendor/util/veac.package.api.json"),
        canonical_api_metadata_json(api).unwrap(),
    )
    .unwrap();
    let text = std::fs::read_to_string(root.join("veac.package.lock")).unwrap();
    let mut lock = parse_package_lock_json(&text).unwrap();
    lock.packages[0].api_sha256 = package_api_digest(api).unwrap();
    std::fs::write(
        root.join("veac.package.lock"),
        canonical_package_lock_json(&lock).unwrap(),
    )
    .unwrap();
}

pub fn rewrite_root_api(root: &Path, api: &ApiMetadataV1) {
    std::fs::write(
        root.join("veac.package.api.json"),
        canonical_api_metadata_json(api).unwrap(),
    )
    .unwrap();
    let text = std::fs::read_to_string(root.join("veac.package.json")).unwrap();
    let mut manifest = parse_package_manifest_json(&text).unwrap();
    manifest.api_sha256 = package_api_digest(api).unwrap();
    std::fs::write(
        root.join("veac.package.json"),
        canonical_package_manifest_json(&manifest).unwrap(),
    )
    .unwrap();
}

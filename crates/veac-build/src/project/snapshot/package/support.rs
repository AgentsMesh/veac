use std::path::Path;

use veac_lang::package::api::{
    canonical_api_metadata_json, from_module_interface, package_api_digest,
};
use veac_lang::package::{
    canonical_package_lock_json, canonical_package_manifest_json, sha256_bytes, ExactVersion,
    LockedFile, PackageIdentity, PackageLockV1, PackageManifestV1, PackageName, Sha256Digest,
};
use veac_lang::program::CompilerDatabase;

use crate::ProjectPackageRevision;

pub(super) fn write_package(root: &Path, name: &str, source: &str) {
    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("main.veac"), source).unwrap();
    let package = identity(name);
    let (loader, entry) =
        veac_lang::program::FileSystemLoader::for_entry(&root.join("main.veac")).unwrap();
    let interface = CompilerDatabase::default()
        .module_interface(entry, &loader)
        .unwrap();
    let api = from_module_interface(package.clone(), &interface);
    let file = LockedFile {
        path: "main.veac".to_owned(),
        sha256: sha256_bytes(source.as_bytes()),
    };
    let manifest = PackageManifestV1::new(
        package.clone(),
        "main.veac",
        file.sha256.clone(),
        Vec::new(),
        package_api_digest(&api).unwrap(),
    );
    let lock = PackageLockV1::new(package, vec![file], Vec::new()).unwrap();
    write(
        &root.join("veac.package.api.json"),
        canonical_api_metadata_json(&api).unwrap(),
    );
    write(
        &root.join("veac.package.json"),
        canonical_package_manifest_json(&manifest).unwrap(),
    );
    write(
        &root.join("veac.package.lock"),
        canonical_package_lock_json(&lock).unwrap(),
    );
}

pub(super) fn identity(name: &str) -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new(name).unwrap(),
        version: ExactVersion::new("1.0.0").unwrap(),
    }
}

pub(super) fn revision(
    name: &str,
    seed: u8,
    dependencies: Vec<PackageIdentity>,
) -> ProjectPackageRevision {
    ProjectPackageRevision {
        package: identity(name),
        entry_module: "main.veac".to_owned(),
        entry_sha256: digest(seed),
        content_sha256: digest(seed.wrapping_add(1)),
        api_sha256: digest(seed.wrapping_add(2)),
        dependencies,
    }
}

fn digest(seed: u8) -> Sha256Digest {
    Sha256Digest::from_bytes([seed; 32])
}

fn write(path: &Path, value: String) {
    std::fs::write(path, format!("{value}\n")).unwrap();
}

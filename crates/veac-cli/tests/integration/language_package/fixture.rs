use std::path::Path;

use veac_lang::package::api::{canonical_api_metadata_json, package_api_digest, ApiMetadataV1};
use veac_lang::package::{
    canonical_package_lock_json, canonical_package_manifest_json, package_content_digest,
    sha256_bytes, ExactVersion, LockedFile, LockedPackage, PackageDependency, PackageIdentity,
    PackageLockV1, PackageManifestV1, PackageName,
};

const SOURCE: &[u8] = b"module {}\n";

pub(super) struct SignedPackage {
    pub(super) content_sha256: String,
    pub(super) dependency_content_sha256: Option<String>,
}

pub(super) fn write_exact_package(
    root: &Path,
    name: &str,
    version: &str,
    with_dependency: bool,
) -> SignedPackage {
    let package = identity(name, version);
    let api = ApiMetadataV1::new(package.clone(), vec![]);
    let root_file = locked_file("main.veac", SOURCE);
    let dependencies = with_dependency.then(|| dependency(name));
    let manifest_dependencies = dependencies
        .as_ref()
        .map(|value| vec![value.dependency.clone()])
        .unwrap_or_default();
    let manifest = PackageManifestV1::new(
        package.clone(),
        "main.veac",
        root_file.sha256.clone(),
        manifest_dependencies,
        package_api_digest(&api).unwrap(),
    );
    let locked_packages = dependencies
        .as_ref()
        .map(|value| vec![value.locked.clone()])
        .unwrap_or_default();
    let lock = PackageLockV1::new(package, vec![root_file], locked_packages).unwrap();

    std::fs::create_dir_all(root).unwrap();
    std::fs::write(root.join("main.veac"), SOURCE).unwrap();
    write_json(
        &root.join("veac.package.api.json"),
        &canonical_api_metadata_json(&api).unwrap(),
    );
    write_json(
        &root.join("veac.package.json"),
        &canonical_package_manifest_json(&manifest).unwrap(),
    );
    write_json(
        &root.join("veac.package.lock"),
        &canonical_package_lock_json(&lock).unwrap(),
    );
    if let Some(value) = &dependencies {
        value.write(root);
    }

    SignedPackage {
        content_sha256: lock.root_content_sha256.to_string(),
        dependency_content_sha256: dependencies
            .map(|value| value.locked.content_sha256.to_string()),
    }
}

#[derive(Clone)]
struct DependencyFixture {
    dependency: PackageDependency,
    locked: LockedPackage,
    api_json: String,
    source: Vec<u8>,
}

impl DependencyFixture {
    fn write(&self, root: &Path) {
        let directory = root.join(&self.locked.path);
        std::fs::create_dir_all(&directory).unwrap();
        std::fs::write(directory.join(&self.locked.entry), &self.source).unwrap();
        write_json(&directory.join("veac.package.api.json"), &self.api_json);
    }
}

fn dependency(root_name: &str) -> DependencyFixture {
    dependency_with(&format!("{root_name}-util"), SOURCE)
}

fn dependency_with(name: &str, source: &[u8]) -> DependencyFixture {
    let package = identity(name, "1.0.0");
    let dependency = PackageDependency {
        name: package.name.clone(),
        version: package.version.clone(),
    };
    let api = ApiMetadataV1::new(package.clone(), vec![]);
    let file = locked_file("lib.veac", source);
    let locked = LockedPackage {
        package,
        path: "vendor/util".to_owned(),
        entry: "lib.veac".to_owned(),
        dependencies: vec![],
        content_sha256: package_content_digest(std::slice::from_ref(&file)).unwrap(),
        files: vec![file],
        api_sha256: package_api_digest(&api).unwrap(),
    };
    DependencyFixture {
        dependency,
        locked,
        api_json: canonical_api_metadata_json(&api).unwrap(),
        source: source.to_vec(),
    }
}

pub(super) fn write_split_brain_pair(owner: &Path, explicit: &Path) {
    write_exact_package(explicit, "shared", "1.0.0", false);
    let package = identity("owner", "1.0.0");
    let api = ApiMetadataV1::new(package.clone(), vec![]);
    let root_file = locked_file("main.veac", SOURCE);
    let dependency = dependency_with("shared", b"module { }\n");
    let manifest = PackageManifestV1::new(
        package.clone(),
        "main.veac",
        root_file.sha256.clone(),
        vec![dependency.dependency.clone()],
        package_api_digest(&api).unwrap(),
    );
    let lock =
        PackageLockV1::new(package, vec![root_file], vec![dependency.locked.clone()]).unwrap();
    std::fs::create_dir_all(owner).unwrap();
    std::fs::write(owner.join("main.veac"), SOURCE).unwrap();
    write_json(
        &owner.join("veac.package.api.json"),
        &canonical_api_metadata_json(&api).unwrap(),
    );
    write_json(
        &owner.join("veac.package.json"),
        &canonical_package_manifest_json(&manifest).unwrap(),
    );
    write_json(
        &owner.join("veac.package.lock"),
        &canonical_package_lock_json(&lock).unwrap(),
    );
    dependency.write(owner);
}

fn identity(name: &str, version: &str) -> PackageIdentity {
    PackageIdentity {
        name: PackageName::new(name).unwrap(),
        version: ExactVersion::new(version).unwrap(),
    }
}

fn locked_file(path: &str, source: &[u8]) -> LockedFile {
    LockedFile {
        path: path.to_owned(),
        sha256: sha256_bytes(source),
    }
}

fn write_json(path: &Path, value: &str) {
    std::fs::write(path, format!("{value}\n")).unwrap();
}

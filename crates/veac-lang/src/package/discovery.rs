use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::api::{package_api_digest, ApiMetadataV1};
use super::fs::BoundDirectory;
use super::{
    PackageError, PackageErrorKind, PackageIdentity, PackageLockV1, Sha256Digest,
    MAX_PACKAGE_CONTENT_BYTES, MAX_PACKAGE_FILE_BYTES, MAX_PACKAGE_JSON_BYTES, PACKAGE_API_FILE,
    PACKAGE_LOCK_FILE, PACKAGE_MANIFEST_FILE,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageDiscovery {
    pub root: DiscoveredPackage,
    pub dependencies: Vec<DiscoveredPackage>,
    pub lock: PackageLockV1,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredPackage {
    pub package: PackageIdentity,
    pub path: PathBuf,
    pub entry: PathBuf,
    pub api: ApiMetadataV1,
    pub entry_sha256: Sha256Digest,
    pub content_sha256: Sha256Digest,
    pub dependencies: Vec<DiscoveredDependency>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoveredDependency {
    pub package: PackageIdentity,
    pub entry: PathBuf,
}

pub fn discover_package(root: &Path) -> Result<PackageDiscovery, PackageError> {
    let root_path = std::fs::canonicalize(root).map_err(|error| {
        PackageError::new(
            PackageErrorKind::Io,
            format!("cannot resolve package root {}: {error}", root.display()),
        )
    })?;
    let directory = BoundDirectory::open(&root_path)?;
    let manifest = super::parse_package_manifest_json(
        &directory.read_text(PACKAGE_MANIFEST_FILE, MAX_PACKAGE_JSON_BYTES)?,
    )?;
    let lock = super::parse_package_lock_json(
        &directory.read_text(PACKAGE_LOCK_FILE, MAX_PACKAGE_JSON_BYTES)?,
    )?;
    if lock.root != manifest.package {
        return contract("package lock root does not match the manifest identity");
    }
    let locked: BTreeMap<_, _> = lock
        .packages
        .iter()
        .map(|item| (item.package.clone(), item))
        .collect();
    super::graph::validate(&manifest.dependencies, &locked)?;
    verify_file_set(&directory, &lock.root_files)?;
    let entry_sha256 = lock
        .root_files
        .iter()
        .find(|file| file.path == manifest.entry)
        .map(|file| file.sha256.clone())
        .ok_or_else(|| error(PackageErrorKind::Contract, "root entry is not locked"))?;
    if entry_sha256 != manifest.entry_sha256 {
        return mismatch("package entry digest does not match the manifest contract");
    }
    let root_api = read_api(&directory, &manifest.package, &manifest.api_sha256)?;
    let root_package = DiscoveredPackage {
        package: manifest.package.clone(),
        entry: root_path.join(&manifest.entry),
        path: root_path.clone(),
        api: root_api,
        entry_sha256,
        content_sha256: lock.root_content_sha256.clone(),
        dependencies: dependency_refs(&manifest.dependencies, &locked, &root_path),
    };
    let mut dependencies = Vec::with_capacity(lock.packages.len());
    for package in &lock.packages {
        let package_root = directory.directory(&package.path)?;
        verify_file_set(&package_root, &package.files)?;
        let api = read_api(&package_root, &package.package, &package.api_sha256)?;
        let path = root_path.join(&package.path);
        dependencies.push(DiscoveredPackage {
            package: package.package.clone(),
            entry: path.join(&package.entry),
            path,
            api,
            entry_sha256: package
                .files
                .iter()
                .find(|file| file.path == package.entry)
                .map(|file| file.sha256.clone())
                .expect("validated lock entry is present"),
            content_sha256: package.content_sha256.clone(),
            dependencies: dependency_refs(&package.dependencies, &locked, &root_path),
        });
    }
    let discovery = PackageDiscovery {
        root: root_package,
        dependencies,
        lock,
    };
    super::interface_verification::verify(&discovery)?;
    Ok(discovery)
}

fn read_api(
    directory: &BoundDirectory,
    expected: &PackageIdentity,
    digest: &Sha256Digest,
) -> Result<ApiMetadataV1, PackageError> {
    let text = directory.read_text(PACKAGE_API_FILE, MAX_PACKAGE_JSON_BYTES)?;
    let api = super::api::parse_api_metadata_json(&text)?;
    if &api.package != expected || &package_api_digest(&api)? != digest {
        return mismatch("package API identity or digest does not match the lock contract");
    }
    Ok(api)
}

fn verify_file_set(
    directory: &BoundDirectory,
    files: &[super::LockedFile],
) -> Result<(), PackageError> {
    let mut bytes = 0usize;
    for file in files {
        let content = directory.read(&file.path, MAX_PACKAGE_FILE_BYTES)?;
        bytes = bytes
            .checked_add(content.len())
            .ok_or_else(|| error(PackageErrorKind::Contract, "package content size overflow"))?;
        if bytes > MAX_PACKAGE_CONTENT_BYTES {
            return contract("package content exceeds the aggregate byte limit");
        }
        if super::sha256_bytes(&content) != file.sha256 {
            return mismatch(format!("content digest mismatch for {}", file.path));
        }
        if file.path.ends_with(".veac") {
            std::str::from_utf8(&content).map_err(|error| {
                PackageError::new(
                    PackageErrorKind::Io,
                    format!("source {} is not UTF-8: {error}", file.path),
                )
            })?;
        }
    }
    Ok(())
}

fn dependency_refs(
    values: &[super::PackageDependency],
    locked: &BTreeMap<PackageIdentity, &super::LockedPackage>,
    root: &Path,
) -> Vec<DiscoveredDependency> {
    values
        .iter()
        .map(|value| {
            let identity = value.identity();
            let package = locked
                .get(&identity)
                .expect("validated lock closure contains every dependency");
            DiscoveredDependency {
                package: identity,
                entry: root.join(&package.path).join(&package.entry),
            }
        })
        .collect()
}

fn contract<T>(message: impl Into<String>) -> Result<T, PackageError> {
    Err(error(PackageErrorKind::Contract, message))
}
fn mismatch<T>(message: impl Into<String>) -> Result<T, PackageError> {
    Err(error(PackageErrorKind::DigestMismatch, message))
}
fn error(kind: PackageErrorKind, message: impl Into<String>) -> PackageError {
    PackageError::new(kind, message)
}

use std::collections::{BTreeMap, BTreeSet};

use super::fs::{BoundDirectory, FileIdentity};
use super::{
    PackageDiscovery, PackageError, PackageIdentity, Sha256Digest, MAX_PACKAGE_FILE_BYTES,
};

mod qualified;

#[derive(Debug)]
pub(super) struct SourceRegistry {
    packages: BTreeMap<PackageIdentity, PackageRecord>,
    sources: BTreeMap<String, LockedSource>,
}

#[derive(Debug)]
struct PackageRecord {
    dependencies: BTreeSet<PackageIdentity>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct LockedSource {
    pub digest: Sha256Digest,
    pub identity: FileIdentity,
    pub locator: String,
    owner: PackageIdentity,
}

impl SourceRegistry {
    pub(super) fn build(
        discovery: &PackageDiscovery,
        directory: &BoundDirectory,
    ) -> Result<Self, PackageError> {
        let mut registry = Self {
            packages: BTreeMap::new(),
            sources: BTreeMap::new(),
        };
        let mut identities = BTreeMap::new();
        registry.register_package(
            directory,
            &mut identities,
            &discovery.root.package,
            "",
            &discovery.root.dependencies,
            &discovery.lock.root_files,
        )?;
        for package in &discovery.lock.packages {
            let dependencies = discovery
                .dependencies
                .iter()
                .find(|value| value.package == package.package)
                .expect("discovery contains every locked package")
                .dependencies
                .as_slice();
            registry.register_package(
                directory,
                &mut identities,
                &package.package,
                &package.path,
                dependencies,
                &package.files,
            )?;
        }
        Ok(registry)
    }

    pub(super) fn source(&self, id: &str) -> Result<&LockedSource, PackageError> {
        self.sources
            .get(id)
            .ok_or_else(|| contract("source is not declared by the package contract"))
    }

    pub(super) fn resolve(&self, importer: &str, requested: &str) -> Result<String, PackageError> {
        let importer_source = self
            .sources
            .get(importer)
            .ok_or_else(|| contract("importer source is not declared by the package contract"))?;
        super::source_path::request(requested)?;
        let id = super::source_path::resolve(importer, requested)?;
        let target = self.source(&id)?;
        if target.owner != importer_source.owner {
            return Err(contract("ordinary imports cannot cross a package boundary"));
        }
        Ok(id)
    }

    pub(super) fn package_source(
        &self,
        package: &PackageIdentity,
        requested: &str,
    ) -> Result<String, PackageError> {
        self.packages
            .get(package)
            .ok_or_else(|| contract("package is not present in the lock"))?;
        super::source_path::request(requested)?;
        let requested = requested.trim_start_matches("./");
        let id = super::source_id::logical(package, requested)?;
        let source = self.source(&id)?;
        if &source.owner != package {
            return Err(contract("source is owned by a different locked package"));
        }
        Ok(id)
    }

    fn register_package(
        &mut self,
        directory: &BoundDirectory,
        identities: &mut BTreeMap<FileIdentity, String>,
        package: &PackageIdentity,
        prefix: &str,
        dependencies: &[super::DiscoveredDependency],
        files: &[super::LockedFile],
    ) -> Result<(), PackageError> {
        let record = PackageRecord {
            dependencies: dependencies
                .iter()
                .map(|value| value.package.clone())
                .collect(),
        };
        if self.packages.insert(package.clone(), record).is_some() {
            return Err(contract("package identity has multiple source owners"));
        }
        for file in files.iter().filter(|file| file.path.ends_with(".veac")) {
            let locator = join(prefix, &file.path);
            let id = super::source_id::logical(package, &file.path)?;
            let (bytes, identity) =
                directory.read_with_identity(&locator, MAX_PACKAGE_FILE_BYTES)?;
            if super::sha256_bytes(&bytes) != file.sha256 {
                return Err(PackageError::new(
                    super::PackageErrorKind::DigestMismatch,
                    format!("source digest mismatch while binding {locator}"),
                ));
            }
            register_identity(identities, identity, &id)?;
            let source = LockedSource {
                digest: file.sha256.clone(),
                identity,
                locator,
                owner: package.clone(),
            };
            if self.sources.insert(id, source).is_some() {
                return Err(contract("source ID has multiple package owners"));
            }
        }
        Ok(())
    }
}

fn join(prefix: &str, path: &str) -> String {
    if prefix.is_empty() {
        path.to_owned()
    } else {
        format!("{prefix}/{path}")
    }
}

fn register_identity(
    identities: &mut BTreeMap<FileIdentity, String>,
    identity: FileIdentity,
    id: &str,
) -> Result<(), PackageError> {
    if let Some(existing) = identities.insert(identity, id.to_owned()) {
        if existing != id {
            return Err(contract(format!(
                "source `{id}` and `{existing}` refer to the same physical file"
            )));
        }
    }
    Ok(())
}

fn contract(message: impl Into<String>) -> PackageError {
    PackageError::new(super::PackageErrorKind::Contract, message)
}

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use super::fs::BoundDirectory;
use super::source_registry::SourceRegistry;
use super::{
    discover_package, sha256_bytes, PackageDiscovery, PackageError, PackageIdentity,
    MAX_PACKAGE_FILE_BYTES,
};
use crate::program::{LoadedSource, SourceAuthority, SourceLoader};

#[derive(Debug)]
pub struct PackageSourceLoader {
    root: PathBuf,
    package: PackageIdentity,
    directory: BoundDirectory,
    entry: String,
    registry: SourceRegistry,
    contracts: BTreeMap<PackageIdentity, super::portable_contract::PortablePackageContract>,
}

impl PackageSourceLoader {
    pub fn for_root(root: &Path) -> Result<(Self, LoadedSource), PackageError> {
        let discovery = discover_package(root)?;
        let loader = Self::from_discovery(&discovery)?;
        let entry = loader.load_entry()?;
        Ok((loader, entry))
    }

    pub(super) fn from_discovery(discovery: &PackageDiscovery) -> Result<Self, PackageError> {
        let root = discovery.root.path.clone();
        let directory = BoundDirectory::open(&root)?;
        let relative = relative_id(&root, &discovery.root.entry)?;
        let registry = SourceRegistry::build(discovery, &directory)?;
        let entry = registry.package_source(&discovery.root.package, &relative)?;
        let contracts = super::portable_contract::closure(discovery)?;
        registry.source(&entry)?;
        Ok(Self {
            root,
            package: discovery.root.package.clone(),
            directory,
            entry,
            registry,
            contracts,
        })
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn identity(&self) -> &PackageIdentity {
        &self.package
    }

    pub fn entry_id(&self) -> &str {
        &self.entry
    }

    pub fn load_entry(&self) -> Result<LoadedSource, PackageError> {
        self.read_id(&self.entry)
    }

    pub fn load_package(
        &self,
        package: &PackageIdentity,
        requested: &str,
    ) -> Result<LoadedSource, PackageError> {
        let id = self.registry.package_source(package, requested)?;
        self.read_id(&id)
    }

    pub fn load_qualified(
        &self,
        importer: &str,
        package: &PackageIdentity,
        requested: &str,
    ) -> Result<LoadedSource, PackageError> {
        let id = self
            .registry
            .qualified_source(importer, package, requested)?;
        self.read_id(&id)
    }

    pub(crate) fn contains_source_id(&self, id: &str) -> bool {
        self.registry.source(id).is_ok()
    }

    pub(crate) fn contracts(
        &self,
    ) -> &BTreeMap<PackageIdentity, super::portable_contract::PortablePackageContract> {
        &self.contracts
    }

    fn read_id(&self, id: &str) -> Result<LoadedSource, PackageError> {
        let source = self.registry.source(id)?;
        let (bytes, identity) = self
            .directory
            .read_with_identity(&source.locator, MAX_PACKAGE_FILE_BYTES)?;
        if identity != source.identity {
            return Err(contract(format!("source identity changed for {id}")));
        }
        if sha256_bytes(&bytes) != source.digest {
            return Err(PackageError::new(
                super::PackageErrorKind::DigestMismatch,
                format!("source digest mismatch for {id}"),
            ));
        }
        let source_text = String::from_utf8(bytes).map_err(|error| {
            PackageError::new(
                super::PackageErrorKind::Io,
                format!("source {id} is not UTF-8: {error}"),
            )
        })?;
        Ok(LoadedSource {
            id: id.to_owned(),
            source: source_text,
        })
    }
}

impl SourceLoader for PackageSourceLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        let resolved = match super::source_id::request(requested) {
            Ok(Some((package, path))) => self.registry.qualified_source(importer, &package, path),
            Ok(None) => self.registry.resolve(importer, requested),
            Err(error) => Err(error),
        };
        resolved
            .and_then(|id| self.read_id(&id))
            .map_err(|error| error.to_string())
    }

    fn authority(&self, _source_id: &str) -> SourceAuthority {
        SourceAuthority::ReadOnlyDependency
    }
}

pub(super) fn relative_id(root: &Path, path: &Path) -> Result<String, PackageError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| contract("discovered package entry escapes its root"))?;
    relative
        .to_str()
        .ok_or_else(|| contract("package entry is not valid UTF-8"))
        .and_then(|value| {
            super::validation::relative_path(value)?;
            Ok(value.replace(std::path::MAIN_SEPARATOR, "/"))
        })
}

fn contract(message: impl Into<String>) -> PackageError {
    PackageError::new(super::PackageErrorKind::Contract, message)
}

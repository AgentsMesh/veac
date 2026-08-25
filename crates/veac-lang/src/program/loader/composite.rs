use std::collections::{btree_map::Entry, BTreeMap};

use crate::package::source_id::{
    label as selector, request as package_request, source as package_source, SOURCE_PREFIX,
};
use crate::package::{PackageIdentity, PackageSourceLoader};

use super::{FileSystemLoader, LoadedSource, SourceAuthority, SourceLoader};

/// 将 project loader 与经过 package trust loop 的 module loader 组合起来。
/// package 请求使用 `package:name@version/path`，返回的 source ID 永远带 package 命名空间。
#[derive(Debug)]
pub struct CompositeSourceLoader {
    project: FileSystemLoader,
    packages: BTreeMap<PackageIdentity, PackageSourceLoader>,
    contracts:
        BTreeMap<PackageIdentity, crate::package::portable_contract::PortablePackageContract>,
}

impl CompositeSourceLoader {
    pub fn new(project: FileSystemLoader) -> Self {
        Self {
            project,
            packages: BTreeMap::new(),
            contracts: BTreeMap::new(),
        }
    }

    pub fn mount_package(
        &mut self,
        identity: PackageIdentity,
        loader: PackageSourceLoader,
    ) -> Result<(), String> {
        if loader.identity() != &identity {
            return Err(format!(
                "mounted package identity {} does not match loader identity {}",
                selector(&identity),
                selector(loader.identity())
            ));
        }
        if loader.entry_id().is_empty() {
            return Err("package entry ID is empty".to_owned());
        }
        if self.packages.contains_key(&identity) {
            return Err("package identity is already mounted".to_owned());
        }
        for (package, contract) in loader.contracts() {
            if self
                .contracts
                .get(package)
                .is_some_and(|existing| existing != contract)
            {
                return Err(format!(
                    "exact package {} has conflicting portable closure contracts",
                    selector(package)
                ));
            }
        }
        let contracts = loader.contracts().clone();
        match self.packages.entry(identity) {
            Entry::Vacant(entry) => {
                entry.insert(loader);
                self.contracts.extend(contracts);
                Ok(())
            }
            Entry::Occupied(_) => unreachable!("duplicate mount checked before validation"),
        }
    }

    pub fn load_package_entry(&self, identity: &PackageIdentity) -> Result<LoadedSource, String> {
        let loader = self
            .packages
            .get(identity)
            .ok_or_else(|| format!("package {} is not mounted", selector(identity)))?;
        let source = loader.load_entry().map_err(|error| error.to_string())?;
        Ok(source)
    }
}

impl SourceLoader for CompositeSourceLoader {
    fn load(&self, importer: &str, requested: &str) -> Result<LoadedSource, String> {
        if let Some((identity, path)) =
            package_request(requested).map_err(|error| error.to_string())?
        {
            return self.load_qualified(importer, &identity, path);
        }
        if package_source(importer)
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return self.loader_for_source(importer)?.load(importer, requested);
        }
        let loaded = self.project.load(importer, requested)?;
        if loaded.id.starts_with(SOURCE_PREFIX) {
            return Err(format!(
                "project source ID `{}` uses the reserved package namespace",
                loaded.id
            ));
        }
        Ok(loaded)
    }

    fn authority(&self, source_id: &str) -> SourceAuthority {
        if source_id.starts_with(SOURCE_PREFIX) {
            SourceAuthority::ReadOnlyDependency
        } else {
            SourceAuthority::Project
        }
    }
}

impl CompositeSourceLoader {
    fn load_qualified(
        &self,
        importer: &str,
        identity: &PackageIdentity,
        path: &str,
    ) -> Result<LoadedSource, String> {
        if package_source(importer)
            .map_err(|error| error.to_string())?
            .is_some()
        {
            return self
                .loader_for_source(importer)?
                .load_qualified(importer, identity, path)
                .map_err(|error| error.to_string());
        }
        self.packages
            .get(identity)
            .ok_or_else(|| format!("package {} is not mounted", selector(identity)))?
            .load_package(identity, path)
            .map_err(|error| error.to_string())
    }

    fn loader_for_source(&self, source_id: &str) -> Result<&PackageSourceLoader, String> {
        self.packages
            .values()
            .find(|loader| loader.contains_source_id(source_id))
            .ok_or_else(|| {
                let label = package_source(source_id)
                    .ok()
                    .flatten()
                    .map(|(identity, _)| selector(&identity))
                    .unwrap_or_else(|| source_id.to_owned());
                format!("package {label} is not mounted")
            })
    }
}

#[cfg(test)]
#[path = "composite_tests.rs"]
mod tests;

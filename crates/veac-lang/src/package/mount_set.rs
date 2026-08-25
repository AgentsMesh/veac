use std::path::{Path, PathBuf};

use super::{discover_package, PackageDiscovery, PackageError, PackageErrorKind};
use crate::program::{CompositeSourceLoader, FileSystemLoader};

mod contract;

#[derive(Debug, Clone)]
pub struct VerifiedPackageMountSet {
    mounts: Vec<VerifiedMount>,
}

#[derive(Debug, Clone)]
pub(super) struct VerifiedMount {
    pub(super) root: PathBuf,
    pub(super) discovery: PackageDiscovery,
}

impl VerifiedPackageMountSet {
    pub fn capture(roots: &[PathBuf]) -> Result<Self, PackageError> {
        let mut mounts = roots
            .iter()
            .map(|root| {
                discover_package(root).map(|discovery| VerifiedMount {
                    root: discovery.root.path.clone(),
                    discovery,
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        mounts.sort_by(|left, right| {
            left.discovery
                .root
                .package
                .cmp(&right.discovery.root.package)
        });
        contract::validate(&mounts)?;
        let captured = Self { mounts };
        captured.revalidate()?;
        Ok(captured)
    }

    pub fn discoveries(&self) -> impl Iterator<Item = &PackageDiscovery> {
        self.mounts.iter().map(|mount| &mount.discovery)
    }

    pub fn host_roots(&self) -> impl Iterator<Item = &Path> {
        self.mounts.iter().map(|mount| mount.root.as_path())
    }

    pub fn revalidate(&self) -> Result<(), PackageError> {
        for mount in &self.mounts {
            let current = discover_package(&mount.root)?;
            if current != mount.discovery || current.root.path != mount.root {
                return Err(contract_error(format!(
                    "package {} changed after mount-set capture",
                    selector(&mount.discovery.root.package)
                )));
            }
        }
        Ok(())
    }

    pub fn loader(&self, project: FileSystemLoader) -> Result<CompositeSourceLoader, PackageError> {
        self.loader_with(project, || {})
    }

    fn loader_with(
        &self,
        project: FileSystemLoader,
        after_preflight: impl FnOnce(),
    ) -> Result<CompositeSourceLoader, PackageError> {
        self.revalidate()?;
        after_preflight();
        let mut loader = CompositeSourceLoader::new(project);
        for mount in &self.mounts {
            let package = super::PackageSourceLoader::from_discovery(&mount.discovery)?;
            package.load_entry()?;
            let identity = package.identity().clone();
            loader
                .mount_package(identity, package)
                .map_err(contract_error)?;
        }
        self.revalidate()?;
        Ok(loader)
    }
}

fn selector(identity: &super::PackageIdentity) -> String {
    format!("{}@{}", identity.name, identity.version)
}

fn contract_error(message: impl Into<String>) -> PackageError {
    PackageError::new(PackageErrorKind::Contract, message)
}

#[cfg(test)]
#[path = "mount_set/tests.rs"]
mod tests;

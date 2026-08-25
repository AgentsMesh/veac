use std::path::{Path, PathBuf};

use veac_lang::package::api::package_api_digest;
use veac_lang::package::{
    DiscoveredPackage, PackageDiscovery, PackageError, VerifiedPackageMountSet,
};
use veac_lang::program::{CompositeSourceLoader, FileSystemLoader};

use crate::{BuildError, BuildResult, ProjectPackageMountRevision, ProjectPackageRevision};

#[derive(Debug, Clone)]
pub struct ProjectPackageSet {
    packages: VerifiedPackageMountSet,
    revision: Vec<ProjectPackageMountRevision>,
}

impl ProjectPackageSet {
    pub fn capture(roots: &[PathBuf]) -> BuildResult<Self> {
        let packages = VerifiedPackageMountSet::capture(roots).map_err(package_error)?;
        let revision = packages
            .discoveries()
            .map(mount_revision)
            .collect::<BuildResult<Vec<_>>>()?;
        Ok(Self { packages, revision })
    }

    pub fn revision(&self) -> &[ProjectPackageMountRevision] {
        &self.revision
    }

    pub fn host_roots(&self) -> impl Iterator<Item = &Path> {
        self.packages.host_roots()
    }

    pub fn require_revision(&self, expected: &[ProjectPackageMountRevision]) -> BuildResult<()> {
        if self.revision == expected {
            Ok(())
        } else {
            Err(BuildError::invalid(
                "project action package mount revision does not match the explicit roots",
            ))
        }
    }

    pub fn revalidate(&self) -> BuildResult<()> {
        self.packages.revalidate().map_err(package_error)
    }

    pub fn loader(&self, project: FileSystemLoader) -> BuildResult<CompositeSourceLoader> {
        self.packages.loader(project).map_err(package_error)
    }
}

fn mount_revision(discovery: &PackageDiscovery) -> BuildResult<ProjectPackageMountRevision> {
    let root = package_revision(&discovery.root)?;
    let mut dependencies = discovery
        .dependencies
        .iter()
        .map(package_revision)
        .collect::<BuildResult<Vec<_>>>()?;
    dependencies.sort_by(|left, right| left.package.cmp(&right.package));
    Ok(ProjectPackageMountRevision { root, dependencies })
}

fn package_revision(value: &DiscoveredPackage) -> BuildResult<ProjectPackageRevision> {
    let mut dependencies = value
        .dependencies
        .iter()
        .map(|dependency| dependency.package.clone())
        .collect::<Vec<_>>();
    dependencies.sort();
    Ok(ProjectPackageRevision {
        package: value.package.clone(),
        entry_module: relative_entry(value)?,
        entry_sha256: value.entry_sha256.clone(),
        content_sha256: value.content_sha256.clone(),
        api_sha256: package_api_digest(&value.api).map_err(package_error)?,
        dependencies,
    })
}

fn relative_entry(value: &DiscoveredPackage) -> BuildResult<String> {
    value
        .entry
        .strip_prefix(&value.path)
        .ok()
        .and_then(Path::to_str)
        .map(|path| path.replace(std::path::MAIN_SEPARATOR, "/"))
        .ok_or_else(|| BuildError::invalid("package entry is not a UTF-8 path inside its root"))
}

fn package_error(error: PackageError) -> BuildError {
    BuildError::invalid(format!("project package contract failed: {error}"))
}

#[cfg(test)]
#[path = "package/tests.rs"]
mod tests;

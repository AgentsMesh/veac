use std::collections::BTreeMap;

use super::{DiscoveredPackage, PackageDiscovery, PackageError, PackageIdentity, Sha256Digest};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct PortablePackageContract {
    entry_module: String,
    entry_sha256: Sha256Digest,
    content_sha256: Sha256Digest,
    api_sha256: Sha256Digest,
    dependencies: Vec<PackageIdentity>,
}

pub(crate) fn closure(
    discovery: &PackageDiscovery,
) -> Result<BTreeMap<PackageIdentity, PortablePackageContract>, PackageError> {
    std::iter::once(&discovery.root)
        .chain(&discovery.dependencies)
        .map(|package| Ok((package.package.clone(), package_contract(package)?)))
        .collect()
}

pub(crate) fn package_contract(
    package: &DiscoveredPackage,
) -> Result<PortablePackageContract, PackageError> {
    let mut dependencies = package
        .dependencies
        .iter()
        .map(|dependency| dependency.package.clone())
        .collect::<Vec<_>>();
    dependencies.sort();
    Ok(PortablePackageContract {
        entry_module: super::source_loader::relative_id(&package.path, &package.entry)?,
        entry_sha256: package.entry_sha256.clone(),
        content_sha256: package.content_sha256.clone(),
        api_sha256: super::api::package_api_digest(&package.api)?,
        dependencies,
    })
}

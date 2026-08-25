use std::collections::{btree_map::Entry, BTreeMap};

use super::{contract_error, VerifiedMount};
use crate::package::{DiscoveredPackage, PackageError};

pub(super) fn validate(mounts: &[VerifiedMount]) -> Result<(), PackageError> {
    let mut roots = BTreeMap::new();
    let mut packages = BTreeMap::new();
    for mount in mounts {
        if roots
            .insert(&mount.discovery.root.package, &mount.root)
            .is_some()
        {
            return Err(contract_error(
                "explicit package roots contain a duplicate exact identity",
            ));
        }
        for package in std::iter::once(&mount.discovery.root).chain(&mount.discovery.dependencies) {
            let contract = crate::package::portable_contract::package_contract(package)?;
            match packages.entry(package.package.clone()) {
                Entry::Occupied(existing) if existing.get() != &contract => {
                    return Err(conflict(package));
                }
                Entry::Occupied(_) => {}
                Entry::Vacant(entry) => {
                    entry.insert(contract);
                }
            }
        }
    }
    for (position, mount) in mounts.iter().enumerate() {
        if mounts[..position]
            .iter()
            .any(|other| mount.root.starts_with(&other.root) || other.root.starts_with(&mount.root))
        {
            return Err(contract_error("explicit package roots must not overlap"));
        }
    }
    Ok(())
}

fn conflict(package: &DiscoveredPackage) -> PackageError {
    contract_error(format!(
        "exact package {}@{} has conflicting content or API contracts",
        package.package.name, package.package.version
    ))
}

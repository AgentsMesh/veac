use std::collections::{BTreeMap, BTreeSet};

use super::{LockedPackage, PackageDependency, PackageError, PackageErrorKind, PackageIdentity};

pub(super) fn validate(
    roots: &[PackageDependency],
    locked: &BTreeMap<PackageIdentity, &LockedPackage>,
) -> Result<(), PackageError> {
    let mut reachable = BTreeSet::new();
    let mut pending: Vec<_> = roots.iter().map(PackageDependency::identity).collect();
    while let Some(identity) = pending.pop() {
        if !reachable.insert(identity.clone()) {
            continue;
        }
        let package = locked.get(&identity).ok_or_else(|| {
            error(
                PackageErrorKind::MissingLockedDependency,
                format!("dependency {identity:?} is not present at its exact locked version"),
            )
        })?;
        pending.extend(package.dependencies.iter().map(PackageDependency::identity));
    }
    if reachable.len() != locked.len() {
        return contract("package lock contains unreachable packages");
    }
    acyclic(locked)
}

fn acyclic(locked: &BTreeMap<PackageIdentity, &LockedPackage>) -> Result<(), PackageError> {
    let mut remaining: BTreeMap<_, _> = locked
        .iter()
        .map(|(identity, package)| (identity.clone(), package.dependencies.len()))
        .collect();
    let mut ready: Vec<_> = remaining
        .iter()
        .filter(|(_, count)| **count == 0)
        .map(|(identity, _)| identity.clone())
        .collect();
    let mut removed = 0;
    while let Some(identity) = ready.pop() {
        removed += 1;
        for (dependent, package) in locked {
            if package
                .dependencies
                .iter()
                .any(|value| value.identity() == identity)
            {
                let count = remaining
                    .get_mut(dependent)
                    .expect("all locked packages have dependency counts");
                *count -= 1;
                if *count == 0 {
                    ready.push(dependent.clone());
                }
            }
        }
    }
    if removed != locked.len() {
        return contract("package dependency graph contains a cycle");
    }
    Ok(())
}

fn contract<T>(message: impl Into<String>) -> Result<T, PackageError> {
    Err(error(PackageErrorKind::Contract, message))
}

fn error(kind: PackageErrorKind, message: impl Into<String>) -> PackageError {
    PackageError::new(kind, message)
}

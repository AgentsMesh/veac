use std::collections::BTreeSet;
use std::path::{Component, Path};

use super::{
    LockedFile, PackageDependency, PackageError, PackageErrorKind, PackageLockV1,
    PackageManifestV1, MAX_LOCKED_PACKAGES, PACKAGE_API_FILE, PACKAGE_LOCK_FILE,
    PACKAGE_LOCK_SCHEMA, PACKAGE_MANIFEST_FILE, PACKAGE_MANIFEST_SCHEMA, PACKAGE_SCHEMA_VERSION,
};

pub(crate) fn manifest(value: &PackageManifestV1) -> Result<(), PackageError> {
    identity(
        &value.schema,
        value.schema_version,
        PACKAGE_MANIFEST_SCHEMA,
        "package manifest",
    )?;
    source_path(&value.entry)?;
    dependencies(&value.dependencies)
}

pub(crate) fn lock(value: &PackageLockV1) -> Result<(), PackageError> {
    identity(
        &value.schema,
        value.schema_version,
        PACKAGE_LOCK_SCHEMA,
        "package lock",
    )?;
    compatibility(&value.compatibility)?;
    file_set(&value.root_files)?;
    if value.root_content_sha256 != super::package_content_digest(&value.root_files)? {
        return contract("root package content digest is not canonical");
    }
    if value.packages.len() > MAX_LOCKED_PACKAGES {
        return contract("package lock exceeds the package limit");
    }
    let mut identities = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut previous = None;
    for package in &value.packages {
        if previous
            .as_ref()
            .is_some_and(|item| item >= &package.package)
        {
            return contract("locked packages must be sorted by exact identity");
        }
        previous = Some(package.package.clone());
        if package.package == value.root || !identities.insert(package.package.clone()) {
            return contract("locked package identities must be unique and exclude the root");
        }
        relative_path(&package.path)?;
        if reserved(&package.path) {
            return contract("locked package paths cannot use package contract names");
        }
        if !paths.insert(package.path.as_str()) {
            return contract("locked package paths must be unique");
        }
        if paths
            .iter()
            .any(|path| is_ancestor(path, &package.path) || is_ancestor(&package.path, path))
        {
            return contract("locked package paths cannot overlap or nest");
        }
        if value.root_files.iter().any(|file| {
            file.path == package.path
                || is_ancestor(&package.path, &file.path)
                || is_ancestor(&file.path, &package.path)
        }) {
            return contract("root files cannot overlap locked dependency package paths");
        }
        source_path(&package.entry)?;
        dependencies(&package.dependencies)?;
        files(&package.files, &package.entry)?;
        if package.content_sha256 != super::package_content_digest(&package.files)? {
            return contract("locked package content digest is not canonical");
        }
    }
    Ok(())
}

fn compatibility(value: &super::PackageCompatibility) -> Result<(), PackageError> {
    let current = super::PackageCompatibility::current();
    if value != &current {
        return contract("package lock targets an incompatible VEAC compiler contract");
    }
    Ok(())
}

pub(crate) fn dependencies(values: &[PackageDependency]) -> Result<(), PackageError> {
    let mut previous = None;
    let mut names = BTreeSet::new();
    for value in values {
        let identity = value.identity();
        if !names.insert(&value.name) {
            return contract("a package may depend on only one exact version of a name");
        }
        if previous.as_ref().is_some_and(|item| item >= &identity) {
            return contract("package dependencies must be unique and sorted by exact identity");
        }
        previous = Some(identity);
    }
    Ok(())
}

pub(crate) fn files(values: &[LockedFile], entry: &str) -> Result<(), PackageError> {
    file_set(values)?;
    values
        .iter()
        .any(|value| value.path == entry)
        .then_some(())
        .ok_or_else(|| error("package entry must be present in locked files"))
}

pub(crate) fn file_set(values: &[LockedFile]) -> Result<(), PackageError> {
    if values.is_empty() || values.len() > super::MAX_PACKAGE_FILES {
        return contract("locked files must be non-empty and within the file limit");
    }
    let mut previous = None;
    for value in values {
        relative_path(&value.path)?;
        if reserved(&value.path) {
            return contract("package contract files cannot be locked content files");
        }
        if previous.is_some_and(|item| item >= value.path.as_str()) {
            return contract("locked files must be unique and sorted by path");
        }
        previous = Some(value.path.as_str());
    }
    Ok(())
}

pub(crate) fn relative_path(value: &str) -> Result<(), PackageError> {
    let path = Path::new(value);
    let valid = !value.is_empty()
        && value.len() <= 4096
        && !value.contains(['\\', ':', '\0'])
        && !value.chars().any(char::is_control)
        && !path.is_absolute()
        && value
            .split('/')
            .all(|part| !part.is_empty() && part != "." && part != "..")
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_)));
    valid.then_some(()).ok_or_else(|| {
        PackageError::new(
            PackageErrorKind::RootEscape,
            "path is not canonical and root-confined",
        )
    })
}

pub(crate) fn source_path(value: &str) -> Result<(), PackageError> {
    relative_path(value)?;
    if !value.ends_with(".veac") || reserved(value) {
        return contract("package entry must be a canonical .veac source path");
    }
    Ok(())
}

fn reserved(value: &str) -> bool {
    Path::new(value)
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            matches!(
                name,
                PACKAGE_MANIFEST_FILE | PACKAGE_LOCK_FILE | PACKAGE_API_FILE
            )
        })
}

fn is_ancestor(parent: &str, child: &str) -> bool {
    child.starts_with(parent) && child.as_bytes().get(parent.len()) == Some(&b'/')
}

fn identity(actual: &str, version: u32, expected: &str, label: &str) -> Result<(), PackageError> {
    if actual != expected || version != PACKAGE_SCHEMA_VERSION {
        return contract(format!("unsupported {label} schema identity"));
    }
    Ok(())
}

fn contract<T>(message: impl Into<String>) -> Result<T, PackageError> {
    Err(error(message))
}
fn error(message: impl Into<String>) -> PackageError {
    PackageError::new(PackageErrorKind::Contract, message)
}

use super::{ExactVersion, PackageError, PackageErrorKind, PackageIdentity, PackageName};

pub(crate) const REQUEST_PREFIX: &str = "package:";
pub(crate) const SOURCE_PREFIX: &str = "packages/";

pub(super) fn logical(package: &PackageIdentity, path: &str) -> Result<String, PackageError> {
    super::validation::source_path(path)?;
    Ok(format!(
        "{SOURCE_PREFIX}{}@{}/{path}",
        package.name, package.version
    ))
}

pub(crate) fn request(value: &str) -> Result<Option<(PackageIdentity, &str)>, PackageError> {
    if !value.starts_with(REQUEST_PREFIX) {
        return Ok(None);
    }
    selector(&value[REQUEST_PREFIX.len()..]).map(Some)
}

pub(crate) fn source(value: &str) -> Result<Option<(PackageIdentity, &str)>, PackageError> {
    if !value.starts_with(SOURCE_PREFIX) {
        return Ok(None);
    }
    selector(&value[SOURCE_PREFIX.len()..]).map(Some)
}

pub(crate) fn selector(value: &str) -> Result<(PackageIdentity, &str), PackageError> {
    let (selector, path) = value
        .split_once('/')
        .ok_or_else(|| contract("package source must include a path"))?;
    let (name, version) = selector
        .split_once('@')
        .ok_or_else(|| contract("package source must use name@version"))?;
    let identity = PackageIdentity {
        name: PackageName::new(name)?,
        version: ExactVersion::new(version)?,
    };
    super::validation::source_path(path)?;
    Ok((identity, path))
}

pub(crate) fn label(identity: &PackageIdentity) -> String {
    format!("{}@{}", identity.name, identity.version)
}

fn contract(message: impl Into<String>) -> PackageError {
    PackageError::new(PackageErrorKind::Contract, message)
}

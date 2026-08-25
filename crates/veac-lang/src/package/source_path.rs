use std::path::{Component, Path};

use super::{PackageError, PackageErrorKind};

pub(super) fn resolve(importer: &str, requested: &str) -> Result<String, PackageError> {
    request(requested)?;
    super::validation::relative_path(importer)?;
    let parent = Path::new(importer)
        .parent()
        .unwrap_or_else(|| Path::new(""));
    let id = parent
        .join(requested)
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => value.to_str().map(str::to_owned),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    super::validation::relative_path(&id)?;
    crate::source_edit::validate_module_path(&id)
        .map_err(|_| contract("package source ID is not a canonical module path"))?;
    Ok(id)
}

pub(super) fn request(value: &str) -> Result<(), PackageError> {
    let path = Path::new(value);
    let valid = !value.is_empty()
        && value.len() <= 4096
        && !value.contains(['\\', ':', '\0'])
        && !value.chars().any(char::is_control)
        && !path.is_absolute()
        && path
            .components()
            .all(|part| matches!(part, Component::Normal(_) | Component::CurDir))
        && path
            .components()
            .any(|part| matches!(part, Component::Normal(_)));
    valid
        .then_some(())
        .ok_or_else(|| contract("package import must be a root-confined relative source path"))
}

fn contract(message: impl Into<String>) -> PackageError {
    PackageError::new(PackageErrorKind::Contract, message)
}

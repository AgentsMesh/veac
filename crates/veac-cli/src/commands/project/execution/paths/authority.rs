use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::error::{CliError, CliResult};

pub(super) fn validate_package_authorities(
    root: &Path,
    paths: &veac_project::ProjectPaths,
    packages: &veac_build::ProjectPackageSet,
) -> CliResult {
    let authorities = [
        (
            declared(root, paths.source_base.as_str(), "project source base")?,
            "source",
        ),
        (
            declared(root, paths.material_root.as_str(), "project material root")?,
            "material",
        ),
        (
            declared(root, paths.build_root.as_str(), "project build root")?,
            "build",
        ),
        (
            declared(root, paths.cache_root.as_str(), "project cache root")?,
            "cache",
        ),
        (
            declared(root, paths.delivery_root.as_str(), "project delivery root")?,
            "delivery",
        ),
    ];
    for package in packages.host_roots() {
        for (authority, name) in &authorities {
            if package.starts_with(authority) || authority.starts_with(package) {
                return Err(CliError::new(
                    "PROJECT_PACKAGE_AUTHORITY",
                    format!(
                        "package root {} overlaps project {name} root",
                        package.display()
                    ),
                ));
            }
        }
    }
    Ok(())
}

fn declared(root: &Path, relative: &str, label: &str) -> CliResult<PathBuf> {
    let relative = Path::new(relative);
    if relative.is_absolute() {
        return Err(super::escape(&root.join(relative), label));
    }
    let mut current = root.to_path_buf();
    let mut missing = false;
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(super::escape(&root.join(relative), label));
        };
        current.push(component);
        if missing {
            continue;
        }
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(CliError::new(
                    "PROJECT_PATH_SYMLINK",
                    format!(
                        "{label} {} uses symlink component {}; project authorities must be unaliased",
                        root.join(relative).display(),
                        current.display()
                    ),
                ));
            }
            Ok(metadata) if !metadata.is_dir() => {
                return Err(CliError::new(
                    "NOT_A_DIRECTORY",
                    format!("{label} {} is not a directory", current.display()),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => missing = true,
            Err(error) => {
                return Err(CliError::new(
                    "PATH_UNAVAILABLE",
                    format!("cannot inspect {label} {}: {error}", current.display()),
                ));
            }
        }
    }
    super::inside(root, current, label)
}

#[cfg(test)]
#[path = "authority_tests.rs"]
mod tests;

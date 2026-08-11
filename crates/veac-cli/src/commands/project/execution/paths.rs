use std::io::ErrorKind;
use std::path::{Component, Path, PathBuf};

use crate::error::{CliError, CliResult};

#[derive(Debug, Clone)]
pub(super) struct ProjectExecutionRoots {
    pub source: PathBuf,
    pub material: PathBuf,
    pub build: PathBuf,
    pub cache: PathBuf,
    pub delivery: PathBuf,
}

impl ProjectExecutionRoots {
    pub(super) fn resolve(root: &Path, paths: &veac_project::ProjectPaths) -> CliResult<Self> {
        let root = crate::fs::canonical_directory(root, "project root")?;
        let resolved = Self {
            source: existing(&root, paths.source_base.as_str(), "project source base")?,
            material: existing(&root, paths.material_root.as_str(), "project material root")?,
            build: writable(&root, paths.build_root.as_str(), "project build root")?,
            cache: writable(&root, paths.cache_root.as_str(), "project cache root")?,
            delivery: writable(&root, paths.delivery_root.as_str(), "project delivery root")?,
        };
        resolved.validate_authorities()?;
        Ok(resolved)
    }

    fn validate_authorities(&self) -> CliResult {
        let roots = [
            (&self.source, "source"),
            (&self.material, "material"),
            (&self.build, "build"),
            (&self.cache, "cache"),
            (&self.delivery, "delivery"),
        ];
        for (position, (root, name)) in roots.iter().enumerate() {
            for (other, other_name) in &roots[..position] {
                if root.starts_with(other) || other.starts_with(root) {
                    return Err(CliError::new(
                        "PROJECT_PATH_AUTHORITY",
                        format!("project {name} root overlaps {other_name} root"),
                    ));
                }
            }
        }
        Ok(())
    }
}

fn existing(root: &Path, relative: &str, label: &str) -> CliResult<PathBuf> {
    let path = crate::fs::canonical_directory(&root.join(relative), label)?;
    inside(root, path, label)
}

fn writable(root: &Path, relative: &str, label: &str) -> CliResult<PathBuf> {
    let path = root.join(relative);
    reject_writable_aliases(root, &path, label)?;
    if let Err(error) = std::fs::create_dir_all(&path) {
        return Err(create_error(&path, label, error));
    }
    let path = crate::fs::canonical_directory(&path, label)?;
    inside(root, path, label)
}

fn reject_writable_aliases(root: &Path, path: &Path, label: &str) -> CliResult {
    let relative = path.strip_prefix(root).map_err(|_| escape(path, label))?;
    let mut current = root.to_path_buf();
    for component in relative.components() {
        let Component::Normal(component) = component else {
            return Err(escape(path, label));
        };
        current.push(component);
        match std::fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(CliError::new(
                    "PROJECT_PATH_SYMLINK",
                    format!(
                        "{label} {} uses symlink component {}; writable roots must be unaliased",
                        path.display(),
                        current.display()
                    ),
                ));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => break,
            Err(error) => return Err(create_error(path, label, error)),
        }
    }
    Ok(())
}

fn create_error(path: &Path, label: &str, error: std::io::Error) -> CliError {
    CliError::new(
        "PROJECT_PATH_CREATE",
        format!("cannot create {label} {}: {error}", path.display()),
    )
}

fn inside(root: &Path, path: PathBuf, label: &str) -> CliResult<PathBuf> {
    if path.starts_with(root) {
        Ok(path)
    } else {
        Err(escape(&path, label))
    }
}

fn escape(path: &Path, label: &str) -> CliError {
    CliError::new(
        "PROJECT_PATH_ESCAPE",
        format!("{label} {} escapes project root", path.display()),
    )
}

#[cfg(test)]
#[path = "paths/tests.rs"]
mod tests;

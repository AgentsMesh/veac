use std::fs;
use std::path::{Path, PathBuf};

use crate::error::{CliError, CliResult};

pub(crate) struct SourceLocation {
    root: PathBuf,
    module: String,
    path: PathBuf,
}

impl SourceLocation {
    pub(crate) fn resolve(source: &Path) -> CliResult<Self> {
        let module = source
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| invalid(source, "source has no UTF-8 file name"))?
            .to_owned();
        veac_lang::source_edit::validate_module_path(&module)
            .map_err(|error| invalid(source, &error.to_string()))?;
        let parent = source
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let root = fs::canonicalize(parent).map_err(|error| {
            CliError::new(
                "PATH_UNAVAILABLE",
                format!("cannot resolve source root {}: {error}", parent.display()),
            )
        })?;
        if !root.is_dir() {
            return Err(invalid(source, "source parent is not a directory"));
        }
        let path = root.join(&module);
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            CliError::new(
                "PATH_UNAVAILABLE",
                format!("cannot inspect source {}: {error}", path.display()),
            )
        })?;
        if !metadata.file_type().is_file() {
            return Err(invalid(source, "source is not a regular file"));
        }
        Ok(Self { root, module, path })
    }

    pub(crate) fn root(&self) -> &Path {
        &self.root
    }

    pub(crate) fn module(&self) -> &str {
        &self.module
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

fn invalid(source: &Path, message: &str) -> CliError {
    CliError::new(
        "INVALID_SOURCE_PATH",
        format!("invalid source path {}: {message}", source.display()),
    )
}

#[cfg(test)]
#[path = "source_location/tests.rs"]
mod tests;

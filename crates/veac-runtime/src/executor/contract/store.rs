use std::ffi::OsString;
use std::fs;
use std::path::{Component, Path, PathBuf};

use super::{invalid, paths::declaration};
use crate::executor::model::RuntimeBundle;
use crate::RuntimeError;

pub(super) fn validate(bundle: &RuntimeBundle, root: &Path) -> Result<(), RuntimeError> {
    let root = normalize_future(root)?;
    for output in declaration::collect(bundle)? {
        if declaration::overlaps_root(&output, &root) {
            return invalid("backend output overlaps the artifact store management directory");
        }
    }
    Ok(())
}

fn normalize_future(path: &Path) -> Result<PathBuf, RuntimeError> {
    if path
        .components()
        .any(|value| matches!(value, Component::ParentDir))
    {
        return invalid("artifact store path may not contain '..'");
    }
    let mut current = path;
    let mut missing = Vec::<OsString>::new();
    while fs::symlink_metadata(current).is_err() {
        let name = current
            .file_name()
            .ok_or_else(|| RuntimeError::new("artifact store has no existing ancestor"))?;
        missing.push(name.to_owned());
        current = current
            .parent()
            .filter(|value| !value.as_os_str().is_empty())
            .unwrap_or(Path::new("."));
    }
    let mut normalized = fs::canonicalize(current).map_err(path_error)?;
    for name in missing.iter().rev() {
        normalized.push(name);
    }
    Ok(normalized)
}

fn path_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!("cannot validate artifact store path: {error}"))
}

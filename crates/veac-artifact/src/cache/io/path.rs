use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

use super::common::unsafe_path;
use crate::ArtifactResult;

pub(super) fn target_parts(
    root: &Path,
    target: &Path,
) -> ArtifactResult<(Vec<OsString>, OsString)> {
    let relative = target
        .strip_prefix(root)
        .map_err(|_| unsafe_path::<()>("cache path escaped root").unwrap_err())?;
    let mut names = normal_names(relative)?;
    let name = names
        .pop()
        .ok_or_else(|| unsafe_path::<()>("cache target has no name").unwrap_err())?;
    Ok((names, name))
}

pub(super) fn path_parts(path: &Path) -> ArtifactResult<(&Path, Vec<OsString>, PathBuf)> {
    let absolute = path.is_absolute();
    let names = normal_names(path)?;
    let base = if absolute {
        Path::new("/")
    } else {
        Path::new(".")
    };
    let normalized = if absolute {
        PathBuf::from("/")
    } else {
        std::env::current_dir()?
    };
    Ok((base, names, normalized))
}

fn normal_names(path: &Path) -> ArtifactResult<Vec<OsString>> {
    let mut names = Vec::new();
    for component in path.components() {
        match component {
            Component::RootDir | Component::CurDir => {}
            Component::Normal(name) => names.push(name.to_owned()),
            Component::ParentDir | Component::Prefix(_) => {
                return unsafe_path("cache paths may not contain parent or prefix components")
            }
        }
    }
    Ok(names)
}

#[cfg(test)]
#[path = "path/tests.rs"]
mod tests;

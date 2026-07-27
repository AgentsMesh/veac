use std::fs;
use std::path::{Path, PathBuf};

use veac_ir::ImageSequencePattern;

use super::{ensure_safe_parent, path_string};
use crate::executor::contract::pattern;
use crate::RuntimeError;

pub(in crate::executor) fn enumerate_pattern(path: &Path) -> Result<Vec<PathBuf>, RuntimeError> {
    let parent = ensure_safe_parent(path)?;
    let name = path.file_name().unwrap().to_string_lossy();
    let Some(pattern) = ImageSequencePattern::parse(&name) else {
        return Err(RuntimeError::new(
            "image sequence output requires exactly one %d or %0Nd placeholder",
        ));
    };
    enumerate(parent, |candidate| pattern.matches(candidate))
}

pub(in crate::executor) fn enumerate_passlogs(path: &Path) -> Result<Vec<PathBuf>, RuntimeError> {
    let parent = ensure_safe_parent(path)?;
    let prefix = path.file_name().unwrap().to_string_lossy();
    enumerate(parent, |candidate| {
        pattern::passlog_accepts(prefix.as_bytes(), candidate.as_bytes())
    })
}

fn enumerate(parent: &Path, matches: impl Fn(&str) -> bool) -> Result<Vec<PathBuf>, RuntimeError> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(parent).map_err(io_error)? {
        let entry = entry.map_err(io_error)?;
        let name = entry.file_name().to_string_lossy().into_owned();
        if !matches(&name) {
            continue;
        }
        let metadata = fs::symlink_metadata(entry.path()).map_err(io_error)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() {
            return Err(RuntimeError::new(format!(
                "matching render output {} is not a regular file",
                entry.path().display()
            )));
        }
        paths.push(entry.path());
    }
    paths.sort_by_key(|path| path_string(path));
    Ok(paths)
}

fn io_error(error: std::io::Error) -> RuntimeError {
    RuntimeError::new(format!("cannot inspect render outputs: {error}"))
}

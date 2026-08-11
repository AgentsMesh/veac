use std::path::{Component, Path};

mod existing;
mod pack;
mod unpack;

pub(super) use pack::pack;
pub(super) use unpack::publish;
#[cfg(test)]
use unpack::unpack;

const MAGIC: &[u8; 8] = b"VEACDIR1";
const ENTRY_END: u8 = 0;
const ENTRY_DIRECTORY: u8 = 1;
const ENTRY_FILE: u8 = 2;
const MAX_ENTRIES: usize = 16_384;
const MAX_PATH_BYTES: usize = 4_096;

fn relative_text(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "directory entry escaped its root".to_owned())?;
    let value = relative
        .to_str()
        .ok_or_else(|| "directory entry path is not UTF-8".to_owned())?;
    if value.is_empty()
        || value.len() > MAX_PATH_BYTES
        || relative
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("directory entry path is not canonical and relative".to_owned());
    }
    Ok(value.replace(std::path::MAIN_SEPARATOR, "/"))
}

fn checked_relative(value: &str) -> Result<&Path, String> {
    let path = Path::new(value);
    if value.is_empty()
        || value.len() > MAX_PATH_BYTES
        || value.contains('\\')
        || path.is_absolute()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
    {
        return Err("directory archive path is not canonical and relative".to_owned());
    }
    Ok(path)
}

#[cfg(test)]
#[path = "directory/tests.rs"]
mod tests;

#[cfg(test)]
#[path = "directory/edge_tests.rs"]
mod edge_tests;

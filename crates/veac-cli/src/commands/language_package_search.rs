use std::path::Path;

use crate::error::{CliError, CliResult};
use veac_lang::package::{DiscoveredPackage, PACKAGE_MANIFEST_FILE};

const MAX_STORE_ENTRIES: usize = 4_096;

pub(super) fn search(store: &Path, query: &str) -> CliResult<Vec<DiscoveredPackage>> {
    let store = crate::fs::canonical_directory(store, "VEAC package store")?;
    validate_query(query)?;
    let entries = match std::fs::read_dir(&store) {
        Ok(entries) => entries,
        Err(error) => return Err(io_error(&store, error)),
    };
    let mut paths = match entries
        .take(MAX_STORE_ENTRIES + 1)
        .collect::<Result<Vec<_>, _>>()
    {
        Ok(paths) => paths,
        Err(error) => return Err(io_error(&store, error)),
    };
    if paths.len() > MAX_STORE_ENTRIES {
        return Err(CliError::resource_limit(
            "LANG_PACKAGE_STORE_LIMIT",
            "local package store exceeds the entry limit",
        ));
    }
    paths.sort_by_key(std::fs::DirEntry::file_name);
    let mut output = Vec::new();
    for entry in paths {
        let file_type = match entry.file_type() {
            Ok(file_type) => file_type,
            Err(error) => return Err(io_error(&store, error)),
        };
        if !file_type.is_dir() || file_type.is_symlink() {
            continue;
        }
        if !manifest_candidate(&entry.path())? {
            continue;
        }
        let discovery = match veac_lang::package::discover_package(&entry.path()) {
            Ok(value) => value,
            Err(error) if error.kind() == veac_lang::package::PackageErrorKind::Io => continue,
            Err(error) => return Err(super::language_package::package_error(error)),
        };
        if discovery.root.package.name.as_str().contains(query) {
            output.push(discovery.root);
        }
    }
    output.sort_by(|left, right| left.package.cmp(&right.package));
    Ok(output)
}

fn manifest_candidate(root: &Path) -> CliResult<bool> {
    match std::fs::symlink_metadata(root.join(PACKAGE_MANIFEST_FILE)) {
        Ok(_) => Ok(true),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(error) => Err(io_error(root, error)),
    }
}

fn validate_query(query: &str) -> CliResult {
    let valid = !query.is_empty()
        && query.len() <= 128
        && query.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || b".-".contains(&byte)
        });
    valid.then_some(()).ok_or_else(|| {
        CliError::new(
            "LANG_PACKAGE_QUERY",
            "package query must be 1-128 lowercase name characters",
        )
    })
}

fn io_error(path: &Path, error: std::io::Error) -> CliError {
    CliError::new(
        "LANG_PACKAGE_STORE_IO",
        format!("cannot read package store {}: {error}", path.display()),
    )
}

#[cfg(test)]
#[path = "language_package_search_tests.rs"]
mod tests;

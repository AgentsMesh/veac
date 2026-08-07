use std::path::{Path, PathBuf};

use crate::error::CliResult;

pub(crate) fn prepare_source_graph(path: &Path) -> CliResult<veac_lang::program::ExecutableBuild> {
    read_source_graph(path, |location| {
        veac_lang::program::prepare_path(location.path())
            .map_err(|errors| crate::diagnostic::program(path, errors))
    })
    .map(|(_, program)| program)
}

pub(crate) fn read_source_graph<T>(
    path: &Path,
    read: impl FnOnce(&crate::fs::SourceLocation) -> CliResult<T>,
) -> CliResult<(PathBuf, T)> {
    let location = crate::fs::SourceLocation::resolve(path)?;
    let lock = crate::fs::SourceGraphReadLock::acquire(location.root())?;
    lock.revalidate(location.root())?;
    let result = read(&location);
    lock.revalidate(location.root())?;
    result.map(|value| (location.root().to_owned(), value))
}

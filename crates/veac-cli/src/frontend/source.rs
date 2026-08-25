use std::path::{Path, PathBuf};

use crate::error::{CliError, CliResult};
use veac_lang::package::PackageErrorKind;
use veac_lang::package::VerifiedPackageMountSet;
use veac_lang::program::{CompositeSourceLoader, FileSystemLoader, LoadedSource};

pub(crate) struct VerifiedSourceLoader {
    pub(crate) entry: LoadedSource,
    pub(crate) loader: CompositeSourceLoader,
    packages: VerifiedPackageMountSet,
}

impl VerifiedSourceLoader {
    pub(crate) fn revalidate_packages(&self) -> CliResult {
        self.packages.revalidate().map_err(package_error)
    }
}
use veac_lang::source_edit::SourceRevision;

pub(crate) fn prepare_source_graph(
    path: &Path,
    package_roots: &[PathBuf],
) -> CliResult<veac_lang::program::ExecutableBuild> {
    read_source_graph(path, package_roots, |_location, entry, loader| {
        veac_lang::program::prepare_with_loader(entry, loader)
            .map_err(|errors| crate::diagnostic::program(path, errors))
    })
    .map(|(_, program)| program)
}

pub(crate) fn source_graph_revision(
    path: &Path,
    package_roots: &[PathBuf],
) -> CliResult<SourceRevision> {
    read_source_graph(path, package_roots, |_location, entry, loader| {
        source_graph_revision_unlocked_with_loader(path, entry, loader)
    })
    .map(|(_, revision)| revision)
}

pub(crate) fn source_loader_unlocked(
    path: &Path,
    package_roots: &[PathBuf],
) -> CliResult<VerifiedSourceLoader> {
    let location = crate::fs::SourceLocation::resolve(path)?;
    source_loader_for_location(&location, package_roots)
}

fn source_loader_for_location(
    location: &crate::fs::SourceLocation,
    package_roots: &[PathBuf],
) -> CliResult<VerifiedSourceLoader> {
    let path = location.path();
    let (project, entry) = FileSystemLoader::for_entry(path).map_err(|message| {
        CliError::new(
            "PROGRAM_ENTRY_LOAD",
            format!("{}: {message}", path.display()),
        )
    })?;
    let project_root = project.root().to_owned();
    let packages = VerifiedPackageMountSet::capture(package_roots).map_err(package_error)?;
    require_distinct_authority(&project_root, &packages)?;
    let loader = packages.loader(project).map_err(package_error)?;
    Ok(VerifiedSourceLoader {
        entry,
        loader,
        packages,
    })
}

fn source_graph_revision_unlocked_with_loader(
    path: &Path,
    entry: LoadedSource,
    loader: &CompositeSourceLoader,
) -> CliResult<SourceRevision> {
    let prepared = veac_lang::program::prepare_with_loader(entry, loader)
        .map_err(|errors| crate::diagnostic::program(path, errors))?;
    prepared
        .source_revision()
        .map_err(|errors| crate::diagnostic::program(path, errors))
}

pub(crate) fn read_source_graph<T>(
    path: &Path,
    package_roots: &[PathBuf],
    read: impl FnOnce(&crate::fs::SourceLocation, LoadedSource, &CompositeSourceLoader) -> CliResult<T>,
) -> CliResult<(PathBuf, T)> {
    let location = crate::fs::SourceLocation::resolve(path)?;
    let lock = crate::fs::SourceGraphReadLock::acquire(location.root())?;
    lock.revalidate(location.root())?;
    let snapshot = source_loader_for_location(&location, package_roots)?;
    let result = read(&location, snapshot.entry.clone(), &snapshot.loader);
    snapshot.revalidate_packages()?;
    lock.revalidate(location.root())?;
    result.map(|value| (location.root().to_owned(), value))
}

fn require_distinct_authority(project: &Path, packages: &VerifiedPackageMountSet) -> CliResult {
    for package in packages.host_roots() {
        if package.starts_with(project) || project.starts_with(package) {
            return Err(CliError::new(
                "LANG_PACKAGE_AUTHORITY",
                format!(
                    "read-only package root {} overlaps the writable project source root",
                    package.display()
                ),
            ));
        }
    }
    Ok(())
}

fn package_error(error: veac_lang::package::PackageError) -> CliError {
    let code = match error.kind() {
        PackageErrorKind::Io => "LANG_PACKAGE_IO",
        PackageErrorKind::Json => "LANG_PACKAGE_JSON",
        PackageErrorKind::Contract => "LANG_PACKAGE_CONTRACT",
        PackageErrorKind::RootEscape => "LANG_PACKAGE_ROOT_ESCAPE",
        PackageErrorKind::MissingLockedDependency => "LANG_PACKAGE_DEPENDENCY",
        PackageErrorKind::DigestMismatch => "LANG_PACKAGE_DIGEST",
    };
    CliError::new(code, error.message())
}

#[cfg(test)]
#[path = "source/tests.rs"]
mod tests;

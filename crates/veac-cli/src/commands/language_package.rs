use std::path::Path;

use crate::arguments::LanguagePackageCommand;
use crate::error::{CliError, CliResult};
use veac_lang::package::{PackageError, PackageErrorKind};

pub(crate) fn run(command: LanguagePackageCommand) -> CliResult {
    match command {
        LanguagePackageCommand::Api { root } => api(&root),
        LanguagePackageCommand::Inspect { root } => inspect(&root),
        LanguagePackageCommand::Search { store, query } => search(&store, &query),
    }
}

fn api(root: &Path) -> CliResult {
    let discovery = veac_lang::package::discover_package(root).map_err(package_error)?;
    write_json(&discovery.root.api)
}

fn inspect(root: &Path) -> CliResult {
    let discovery = veac_lang::package::discover_package(root).map_err(package_error)?;
    write_json(&super::language_package_model::Inspection::from(discovery))
}

fn search(store: &Path, query: &str) -> CliResult {
    let matches = super::language_package_search::search(store, query)?;
    write_json(&super::language_package_model::SearchResult::new(
        store, query, matches,
    ))
}

fn write_json(value: &impl serde::Serialize) -> CliResult {
    let mut json = match serde_json_canonicalizer::to_string(value) {
        Ok(json) => json,
        Err(error) => return Err(CliError::new("LANG_PACKAGE_ENCODE", error.to_string())),
    };
    json.push('\n');
    crate::fs::write_stdout(&json)
}

pub(super) fn package_error(error: PackageError) -> CliError {
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

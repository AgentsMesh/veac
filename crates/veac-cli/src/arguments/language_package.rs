use std::path::PathBuf;

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum LanguagePackageCommand {
    Api { root: PathBuf },
    Inspect { root: PathBuf },
    Search { store: PathBuf, query: String },
}

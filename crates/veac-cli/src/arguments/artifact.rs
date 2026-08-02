use std::path::PathBuf;

#[derive(Debug)]
pub(crate) enum ArtifactCommand {
    Inspect {
        store: PathBuf,
        key: String,
        output: Option<PathBuf>,
    },
    Materialize {
        store: PathBuf,
        key: String,
        destination: PathBuf,
    },
    Remove {
        store: PathBuf,
        key: String,
    },
}

#[derive(Debug)]
pub(crate) struct PackageBindingsArgs {
    pub package: PathBuf,
    pub output: Option<PathBuf>,
}

#[derive(Debug)]
pub(crate) struct RelinkArgs {
    pub project: PathBuf,
    pub config: Option<String>,
    pub search: Vec<PathBuf>,
    pub output: Option<PathBuf>,
}
